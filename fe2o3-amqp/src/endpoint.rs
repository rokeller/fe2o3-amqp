//! Trait abstraction of Connection, Session and Link
//!
//! Frame          Connection  Session  Link
//! ========================================
//! open               H
//! begin              I          H
//! attach                        I       H
//! flow                          I       H
//! transfer                      I       H
//! disposition                   I       H
//! detach                        I       H
//! end                I          H
//! close              H
//! ----------------------------------------
//! Key:
//!     H: handled by the endpoint
//!     I: intercepted (endpoint examines
//!         the frame, but delegates
//!         further processing to another
//!         endpoint)

use async_trait::async_trait;
use bytes::BytesMut;
use fe2o3_amqp_types::{
    definitions::{Error, Handle, Role},
    performatives::{Attach, Begin, Close, Detach, Disposition, End, Flow, Open, Transfer},
};
use futures_util::Sink;
use tokio::sync::mpsc::Sender;

use crate::{
    connection::engine::SessionId,
    error::EngineError,
    link::{LinkFrame, LinkIncomingItem},
    session::{SessionFrame, SessionIncomingItem},
    transport::amqp::Frame,
};

#[async_trait]
pub trait Connection {
    type Error: Into<EngineError> + Send;
    type State: Send;
    type Session: Session + Send;

    fn local_state(&self) -> &Self::State;
    fn local_state_mut(&mut self) -> &mut Self::State;
    fn local_open(&self) -> &Open;

    // Allocate outgoing channel id and session id to a new session
    fn create_session(
        &mut self,
        tx: Sender<SessionIncomingItem>,
    ) -> Result<(u16, SessionId), Self::Error>;
    // Remove outgoing id and session id association
    fn drop_session(&mut self, session_id: usize);

    // async fn forward_to_session(&mut self, incoming_channel: u16, frame: SessionFrame) -> Result<(), Self::Error>;

    /// Reacting to remote Open frame
    async fn on_incoming_open(&mut self, channel: u16, open: Open) -> Result<(), Self::Error>;

    /// Reacting to remote Begin frame
    ///
    /// Do NOT forward to session here. Forwarding is handled elsewhere.
    async fn on_incoming_begin(&mut self, channel: u16, begin: Begin) -> Result<(), Self::Error>;

    /// Reacting to remote End frame
    async fn on_incoming_end(&mut self, channel: u16, end: End) -> Result<(), Self::Error>;

    /// Reacting to remote Close frame
    async fn on_incoming_close(&mut self, channel: u16, close: Close) -> Result<(), Self::Error>;

    /// Sending out an Open frame
    ///
    /// The write is passed in is because sending an Open frame also changes the local
    /// connection state. If the sending fails outside, coming back
    /// and revert the state changes would be too complicated
    async fn on_outgoing_open<W>(&mut self, writer: &mut W) -> Result<(), Self::Error>
    where
        W: Sink<Frame, Error = EngineError> + Send + Unpin;

    async fn on_outgoing_close<W>(
        &mut self,
        writer: &mut W,
        error: Option<Error>,
    ) -> Result<(), Self::Error>
    where
        W: Sink<Frame, Error = EngineError> + Send + Unpin;

    /// Intercepting and outgoing Begin frame
    async fn on_outgoing_begin(&mut self, channel: u16, begin: Begin)
        -> Result<Frame, Self::Error>;

    async fn on_outgoing_end(&mut self, channel: u16, end: End) -> Result<Frame, Self::Error>;

    fn session_tx_by_incoming_channel(
        &mut self,
        channel: u16,
    ) -> Option<&mut Sender<SessionIncomingItem>>;

    fn session_tx_by_outgoing_channel(
        &mut self,
        channel: u16,
    ) -> Option<&mut Sender<SessionIncomingItem>>;
}

#[async_trait]
pub trait Session {
    type Error: Into<EngineError>;
    type State;

    fn local_state(&self) -> &Self::State;
    fn local_state_mut(&mut self) -> &mut Self::State;

    // Allocate new local handle for new Link
    fn create_link(&mut self, tx: Sender<LinkIncomingItem>) -> Result<Handle, EngineError>;
    fn drop_link(&mut self, handle: Handle);

    async fn on_incoming_begin(&mut self, channel: u16, begin: Begin) -> Result<(), Self::Error>;
    async fn on_incoming_attach(&mut self, channel: u16, attach: Attach)
        -> Result<(), Self::Error>;
    async fn on_incoming_flow(&mut self, channel: u16, flow: Flow) -> Result<(), Self::Error>;
    async fn on_incoming_transfer(
        &mut self,
        channel: u16,
        transfer: Transfer,
        payload: Option<BytesMut>,
    ) -> Result<(), Self::Error>;
    async fn on_incoming_disposition(
        &mut self,
        channel: u16,
        disposition: Disposition,
    ) -> Result<(), Self::Error>;
    async fn on_incoming_detach(&mut self, channel: u16, detach: Detach)
        -> Result<(), Self::Error>;
    async fn on_incoming_end(&mut self, channel: u16, end: End) -> Result<(), Self::Error>;

    async fn on_outgoing_begin(
        &mut self,
        writer: &mut Sender<SessionFrame>,
    ) -> Result<(), Self::Error>;
    async fn on_outgoing_end(
        &mut self,
        writer: &mut Sender<SessionFrame>,
        error: Option<Error>,
    ) -> Result<(), Self::Error>;

    async fn on_outgoing_attach(&mut self, attach: Attach) -> Result<SessionFrame, Self::Error>;
    async fn on_outgoing_flow(&mut self, flow: Flow) -> Result<SessionFrame, Self::Error>;
    async fn on_outgoing_transfer(
        &mut self,
        transfer: Transfer,
        payload: Option<BytesMut>,
    ) -> Result<SessionFrame, Self::Error>;
    async fn on_outgoing_disposition(
        &mut self,
        disposition: Disposition,
    ) -> Result<SessionFrame, Self::Error>;
    async fn on_outgoing_detach(&mut self, detach: Detach) -> Result<SessionFrame, Self::Error>;
}

#[async_trait]
pub trait Link {
    type Error: Into<EngineError>;

    async fn on_incoming_attach(&mut self, attach: Attach) -> Result<(), Self::Error>;
    async fn on_incoming_flow(&mut self, flow: Flow) -> Result<(), Self::Error>;
    async fn on_incoming_disposition(
        &mut self,
        disposition: Disposition,
    ) -> Result<(), Self::Error>;
    async fn on_incoming_detach(&mut self, detach: Detach) -> Result<(), Self::Error>;

    async fn on_outgoing_attach(
        &mut self,
        writer: &mut Sender<LinkFrame>,
    ) -> Result<(), Self::Error>;
    async fn on_outgoing_detach(
        &mut self,
        writer: &mut Sender<LinkFrame>,
    ) -> Result<(), Self::Error>;

    // async fn on_incoming_transfer(&mut self, transfer: Transfer, payload: Option<BytesMut>) -> Result<(), Self::Error>;
    // async fn on_outgoing_flow() -> Result<(), Self::Error>;
    // async fn on_outgoing_transfer() -> Result<(), Self::Error>;
    // async fn on_outgoing_disposition() -> Result<(), Self::Error>;
}

pub trait SenderLink: Link {
    const ROLE: Role = Role::Sender;
}

#[async_trait]
pub trait ReceiverLink: Link {
    const ROLE: Role = Role::Receiver;

    async fn on_incoming_transfer(
        &mut self,
        transfer: Transfer,
        payload: Option<BytesMut>,
    ) -> Result<(), <Self as Link>::Error>;
}
