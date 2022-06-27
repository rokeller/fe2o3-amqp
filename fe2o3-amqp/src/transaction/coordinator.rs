//! Control link coordinator

use fe2o3_amqp_types::{
    definitions::{self, AmqpError, DeliveryNumber, DeliveryTag, ErrorCondition, LinkError},
    messaging::{Accepted, DeliveryState, Rejected},
    performatives::Attach,
    transaction::{Coordinator, Declare, Declared, Discharge, TransactionError, TxnCapability},
};
use tokio::sync::mpsc;
use tracing::{instrument};

use crate::{
    acceptor::{link::SharedLinkAcceptorFields, local_receiver_link::LocalReceiverLinkAcceptor},
    control::SessionControl,
    link::{
        receiver::ReceiverInner,
        role,
        shared_inner::{LinkEndpointInner, LinkEndpointInnerDetach},
        IllegalLinkStateError, Link, LinkFrame, ReceiverAttachError, ReceiverFlowState, RecvError,
    },
    util::Running,
    Delivery,
};

use super::{control_link_frame::ControlMessageBody, CoordinatorError};

pub(crate) type CoordinatorLink =
    Link<role::Receiver, Coordinator, ReceiverFlowState, DeliveryState>;

/// An acceptor that handles incoming control links
#[derive(Debug, Clone)]
pub struct ControlLinkAcceptor {
    shared: SharedLinkAcceptorFields,
    inner: LocalReceiverLinkAcceptor<TxnCapability>,
}

impl Default for ControlLinkAcceptor {
    fn default() -> Self {
        Self {
            shared: Default::default(),
            inner: Default::default(),
        }
    }
}

impl ControlLinkAcceptor {
    #[instrument(skip_all)]
    pub(crate) async fn accept_incoming_attach(
        &self,
        remote_attach: Attach,
        control: mpsc::Sender<SessionControl>,
        outgoing: mpsc::Sender<LinkFrame>,
    ) -> Result<TxnCoordinator, ReceiverAttachError> {
        tracing::info!(control_link_attach = ?remote_attach);
        self.inner
            .accept_incoming_attach_inner(&self.shared, remote_attach, control, outgoing)
            .await
            .map(|inner| TxnCoordinator { inner })
    }
}

enum SuccessfulOutcome {
    Declared(Declared),
    Accepted(Accepted),
}

impl From<SuccessfulOutcome> for DeliveryState {
    fn from(value: SuccessfulOutcome) -> Self {
        match value {
            SuccessfulOutcome::Declared(declared) => DeliveryState::Declared(declared),
            SuccessfulOutcome::Accepted(accepted) => DeliveryState::Accepted(accepted),
        }
    }
}

/// Transaction coordinator
#[derive(Debug)]
pub(crate) struct TxnCoordinator {
    inner: ReceiverInner<CoordinatorLink>,
}

impl TxnCoordinator {
    async fn on_declare(&mut self, declare: &Declare) -> Result<Declared, CoordinatorError> {
        match declare.global_id {
            Some(_) => Err(CoordinatorError::GlobalIdNotImplemented),
            None => {
                let txn_id =
                    super::session::allocate_transaction_id(self.inner.session_control()).await?;
                Ok(Declared { txn_id })
            }
        }
    }

    async fn on_discharge(&mut self, discharge: &Discharge) -> Result<Accepted, CoordinatorError> {
        match discharge.fail {
            Some(true) => super::session::rollback_transaction(
                self.inner.session_control(),
                discharge.txn_id.clone(),
            )
            .await
            .map_err(Into::into),
            Some(false) | None => {
                // The fail field is treated as a false if unset in AmqpNetLite
                super::session::commit_transaction(
                    self.inner.session_control(),
                    discharge.txn_id.clone(),
                )
                .await
                .map_err(Into::into)
            }
        }
    }

    async fn on_delivery(&mut self, delivery: Delivery<ControlMessageBody>) -> Running {
        let body = match delivery.body() {
            fe2o3_amqp_types::messaging::Body::Value(v) => &v.0,
            fe2o3_amqp_types::messaging::Body::Sequence(_)
            | fe2o3_amqp_types::messaging::Body::Data(_) => {
                // Message Decode Error?
                let error = definitions::Error::new(
                    AmqpError::DecodeError,
                    "The coordinator is only expecting Declare or Discharge frames".to_string(),
                    None
                );
                // TODO: detach instead of closing
                let _ = self.inner.close_with_error(Some(error)).await;
                return Running::Stop
            }
        };

        let result = match body {
            ControlMessageBody::Declare(declare) => {
                self.on_declare(declare).await.map(SuccessfulOutcome::Declared)
            }
            ControlMessageBody::Discharge(discharge) => self
                .on_discharge(discharge)
                .await
                .map(SuccessfulOutcome::Accepted),
        };

        self.handle_delivery_result(delivery.delivery_id, delivery.delivery_tag, result).await
    }

    async fn on_recv_error(&mut self, error: RecvError) -> Running {
        tracing::error!(?error);

        match error {
            RecvError::LinkStateError(error) => match error {
                crate::link::LinkStateError::IllegalState => {
                    let error =
                        definitions::Error::new(AmqpError::IllegalState, None, None);
                    // TODO: detach instead of closing
                    let _ = self.inner.close_with_error(Some(error)).await;
                    Running::Stop
                },
                crate::link::LinkStateError::IllegalSessionState => {
                    // Session must have already stopped
                    Running::Stop
                },
                crate::link::LinkStateError::ExpectImmediateDetach => {
                    let _ = self.inner.close_with_error(None).await;
                    // TODO: detach instead of closing
                    Running::Stop
                },
                crate::link::LinkStateError::RemoteDetached
                | crate::link::LinkStateError::RemoteDetachedWithError(_)
                | crate::link::LinkStateError::RemoteClosed
                | crate::link::LinkStateError::RemoteClosedWithError(_) => {
                    self.inner
                        .send_detach(true, None)
                        .await
                        .unwrap_or_else(|err| tracing::error!(error = ?err));
                    Running::Stop
                }
            },
            RecvError::TransferLimitExceeded => {
                let error = definitions::Error::new(LinkError::TransferLimitExceeded, None, None);
                // TODO: detach instead of closing
                let _ = self.inner.close_with_error(Some(error)).await;
                Running::Stop
            },
            RecvError::DeliveryIdIsNone 
            | RecvError::DeliveryTagIsNone 
            | RecvError::MessageDecodeError 
            | RecvError::IllegalRcvSettleModeInTransfer 
            | RecvError::InconsistentFieldInMultiFrameDelivery => {
                let error = definitions::Error::new(
                    AmqpError::NotAllowed,
                    format!("{:?}", error),
                    None
                );
                // TODO: detach instead of closing
                let _ = self.inner.close_with_error(Some(error)).await;
                Running::Stop
            },
        }
    }

    async fn handle_delivery_result(
        &mut self, 
        delivery_id: DeliveryNumber,
        delivery_tag: DeliveryTag,
        result: Result<SuccessfulOutcome, CoordinatorError>
    ) -> Running {
        let disposition_result = match result {
            Ok(outcome) => self.inner.dispose(delivery_id, delivery_tag, outcome.into()).await,
            Err(error) => match error {
                CoordinatorError::GlobalIdNotImplemented => {
                    let error = TransactionError::UnknownId;
                    let description = "Global transaction ID is not implemented".to_string();
                    self.reject(delivery_id, delivery_tag, error, description).await
                },
                CoordinatorError::InvalidSessionState => {
                    // Session must have dropped
                    return Running::Stop
                },
                CoordinatorError::AllocTxnIdNotImplemented => {
                    let error = TransactionError::UnknownId;
                    let description = "Allocation of new transaction ID is not implemented".to_string();
                    self.reject(delivery_id, delivery_tag, error, description).await
                },
                CoordinatorError::TransactionError(error) => {
                    self.reject(delivery_id, delivery_tag, error, None).await
                }
            },
        };

        match disposition_result {
            Ok(_) => Running::Continue,
            Err(disposition_error) => match disposition_error {
                IllegalLinkStateError::IllegalState => {
                    let error =
                        definitions::Error::new(AmqpError::IllegalState, None, None);
                    // TODO: detach instead of closing
                    let _ = self.inner.close_with_error(Some(error)).await;
                    Running::Stop
                }
                IllegalLinkStateError::IllegalSessionState => {
                    // Session must have already dropped
                    Running::Stop
                }
            },
        }
    }

    async fn reject(
        &mut self,
        delivery_id: DeliveryNumber,
        delivery_tag: DeliveryTag,
        error: TransactionError,
        description: impl Into<Option<String>>
    ) -> Result<(), IllegalLinkStateError> {
        let condition = ErrorCondition::TransactionError(error);
        let error = definitions::Error::new(condition, description, None);
        let state = DeliveryState::Rejected(Rejected { error: Some(error) });

        self
            .inner
            .dispose(delivery_id, delivery_tag, state)
            .await
    }

    #[instrument(skip_all)]
    pub async fn event_loop(mut self) {
        tracing::info!("Coordinator started");
        loop {
            let running = match self.inner.recv().await {
                Ok(delivery) => self.on_delivery(delivery).await,
                Err(error) => self.on_recv_error(error).await,
            };

            if let Running::Stop = running {
                break;
            }
        }
    }
}
