//! The engine handles incoming and outgoing frames and messages to reduce
//! transferring frames/messages over channels

use std::cmp::min;
use std::io;
use std::time::Duration;

use fe2o3_amqp_types::definitions::{self, AmqpError};
use futures_util::{SinkExt, StreamExt};
use tokio::io::{AsyncRead, AsyncWrite};
use tokio::sync::mpsc::Receiver;
use tokio::task::JoinHandle;

use crate::control::ConnectionControl;
use crate::frames::amqp::{self, Frame};
use crate::session::{SessionFrame, SessionFrameBody};
use crate::transport::Transport;
use crate::util::Running;
use crate::{endpoint, transport};

use super::AllocSessionError;
use super::{heartbeat::HeartBeat, ConnectionState, Error};

pub(crate) type SessionId = usize;

pub(crate) struct ConnectionEngine<Io, C> {
    transport: Transport<Io, amqp::Frame>,
    connection: C,
    control: Receiver<ConnectionControl>,
    outgoing_session_frames: Receiver<SessionFrame>,
    // session_control: Receiver<SessionControl>,
    heartbeat: HeartBeat,
    remote_err: Option<definitions::Error>, // TODO: how to present this back to the user?
}

impl<Io, C> ConnectionEngine<Io, C>
where
    Io: AsyncRead + AsyncWrite + Send + Unpin + 'static,
    C: endpoint::Connection<State = ConnectionState> + Send + 'static,
    C::Error: Into<Error> + From<transport::Error>,
    C::AllocError: Into<AllocSessionError>,
{
    /// Open Connection without starting the Engine::event_loop()
    pub(crate) async fn open(
        transport: Transport<Io, amqp::Frame>,
        connection: C,
        control: Receiver<ConnectionControl>,
        outgoing_session_frames: Receiver<SessionFrame>,
        // session_control: Receiver<SessionControl>,
    ) -> Result<Self, Error> {
        use crate::frames::amqp::FrameBody;

        let mut engine = Self {
            transport,
            connection,
            control,
            outgoing_session_frames,
            heartbeat: HeartBeat::never(),
            remote_err: None,
        };

        // Send an Open
        engine
            .connection
            .send_open(&mut engine.transport)
            .await
            .map_err(Into::into)?;

        // Wait for an Open
        let frame = match engine.transport.next().await {
            Some(frame) => frame?,
            None => todo!(),
        };
        let Frame { channel, body } = frame;
        let remote_open = match body {
            FrameBody::Open(open) => open,
            _ => return Err(AmqpError::IllegalState.into()),
        };

        // Handle incoming remote_open
        let remote_max_frame_size = remote_open.max_frame_size.0;
        let remote_idle_timeout = remote_open.idle_time_out;
        engine
            .connection
            .on_incoming_open(channel, remote_open)
            .await
            .map_err(Into::into)?;

        // update transport setting
        let max_frame_size = min(
            engine.connection.local_open().max_frame_size.0,
            remote_max_frame_size,
        );
        engine.transport.set_max_frame_size(max_frame_size as usize);

        // Set heartbeat here because in pipelined-open, the Open frame
        // may be recved after mux loop is started
        match &remote_idle_timeout {
            Some(millis) => {
                let period = Duration::from_millis(*millis as u64);
                engine.heartbeat = HeartBeat::new(period);
            }
            None => engine.heartbeat = HeartBeat::never(),
        };

        Ok(engine)
    }

    pub fn spawn(self) -> JoinHandle<Result<(), Error>> {
        tokio::spawn(self.event_loop())
    }
}

impl<Io, C> ConnectionEngine<Io, C>
where
    Io: AsyncRead + AsyncWrite + Send + Unpin,
    C: endpoint::Connection<State = ConnectionState> + Send + 'static,
    C::Error: Into<Error> + From<transport::Error>,
    C::AllocError: Into<AllocSessionError>,
{
    async fn forward_to_session(&mut self, channel: u16, frame: SessionFrame) -> Result<(), Error> {
        match &self.connection.local_state() {
            ConnectionState::Opened => {}
            _ => return Err(AmqpError::IllegalState.into()),
        };

        match self.connection.session_tx_by_incoming_channel(channel) {
            Some(tx) => tx.send(frame).await?,
            None => return Err(AmqpError::NotFound.into()),
        };
        Ok(())
    }

    async fn on_incoming(&mut self, incoming: Result<Frame, Error>) -> Result<Running, Error> {
        use crate::frames::amqp::FrameBody;

        let frame = incoming?;

        let Frame { channel, body } = frame;

        match body {
            FrameBody::Open(open) => {
                let remote_max_frame_size = open.max_frame_size.0;
                let remote_idle_timeout = open.idle_time_out;
                self.connection
                    .on_incoming_open(channel, open)
                    .await
                    .map_err(Into::into)?;

                // update transport setting
                let max_frame_size = min(
                    self.connection.local_open().max_frame_size.0,
                    remote_max_frame_size,
                );
                self.transport.set_max_frame_size(max_frame_size as usize);

                // Set heartbeat here because in pipelined-open, the Open frame
                // may be recved after mux loop is started
                match &remote_idle_timeout {
                    Some(millis) => {
                        let period = Duration::from_millis(*millis as u64);
                        self.heartbeat = HeartBeat::new(period);
                    }
                    None => self.heartbeat = HeartBeat::never(),
                };
            }
            FrameBody::Begin(begin) => {
                self.connection
                    .on_incoming_begin(channel, begin)
                    .await
                    .map_err(Into::into)?;
            }
            FrameBody::Attach(attach) => {
                let sframe = SessionFrame::new(channel, SessionFrameBody::Attach(attach));
                self.forward_to_session(channel, sframe).await?;
            }
            FrameBody::Flow(flow) => {
                let sframe = SessionFrame::new(channel, SessionFrameBody::Flow(flow));
                self.forward_to_session(channel, sframe).await?;
            }
            FrameBody::Transfer {
                performative,
                payload,
            } => {
                let sframe = SessionFrame::new(
                    channel,
                    SessionFrameBody::Transfer {
                        performative,
                        payload,
                    },
                );
                self.forward_to_session(channel, sframe).await?;
            }
            FrameBody::Disposition(disposition) => {
                let sframe = SessionFrame::new(channel, SessionFrameBody::Disposition(disposition));
                self.forward_to_session(channel, sframe).await?;
            }
            FrameBody::Detach(detach) => {
                let sframe = SessionFrame::new(channel, SessionFrameBody::Detach(detach));
                self.forward_to_session(channel, sframe).await?;
            }
            FrameBody::End(end) => {
                self.connection
                    .on_incoming_end(channel, end)
                    .await
                    .map_err(Into::into)?;
            }
            FrameBody::Close(close) => {
                self.remote_err = self
                    .connection
                    .on_incoming_close(channel, close)
                    .await
                    .map_err(Into::into)?;
            }
            FrameBody::Empty => {
                // do nothing, IdleTimeout is tracked by Transport
            }
        }

        match self.connection.local_state() {
            ConnectionState::End => Ok(Running::Stop),
            _ => Ok(Running::Continue),
        }
    }

    #[inline]
    async fn on_control(&mut self, control: ConnectionControl) -> Result<Running, Error> {
        println!(">>> Debug: ConectionEnginer::on_control");
        match control {
            ConnectionControl::Open => {
                // let open = self.connection.local_open().clone();
                self.connection
                    .send_open(&mut self.transport)
                    .await
                    .map_err(Into::into)?;
            }
            ConnectionControl::Close(error) => {
                self.connection
                    .send_close(&mut self.transport, error)
                    .await
                    .map_err(Into::into)?;
            }
            ConnectionControl::AllocateSession { tx, responder } => {
                let result = self.connection.allocate_session(tx).map_err(Into::into);
                responder.send(result).map_err(|_| {
                    Error::Io(io::Error::new(
                        io::ErrorKind::Other,
                        "ConnectionHandle is dropped",
                    ))
                })?;
            }
            ConnectionControl::DeallocateSession(session_id) => {
                self.connection.deallocate_session(session_id)
            }
        }

        match self.connection.local_state() {
            ConnectionState::End => Ok(Running::Stop),
            _ => Ok(Running::Continue),
        }
    }

    #[inline]
    async fn on_outgoing_session_frames(&mut self, frame: SessionFrame) -> Result<Running, Error> {
        use crate::frames::amqp::FrameBody;

        match self.connection.local_state() {
            ConnectionState::Opened => {}
            _ => return Err(AmqpError::IllegalState.into()),
        }

        let SessionFrame { channel, body } = frame;

        let frame = match body {
            SessionFrameBody::Begin(begin) => self
                .connection
                .on_outgoing_begin(channel, begin)
                .map_err(Into::into)?,
            SessionFrameBody::Attach(attach) => Frame::new(channel, FrameBody::Attach(attach)),
            SessionFrameBody::Flow(flow) => Frame::new(channel, FrameBody::Flow(flow)),
            SessionFrameBody::Transfer {
                performative,
                payload,
            } => Frame::new(
                channel,
                FrameBody::Transfer {
                    performative,
                    payload,
                },
            ),
            SessionFrameBody::Disposition(disposition) => {
                Frame::new(channel, FrameBody::Disposition(disposition))
            }
            SessionFrameBody::Detach(detach) => Frame::new(channel, FrameBody::Detach(detach)),
            SessionFrameBody::End(end) => self
                .connection
                .on_outgoing_end(channel, end)
                .map_err(Into::into)?,
        };

        self.transport.send(frame).await?;
        Ok(Running::Continue)
    }

    #[inline]
    async fn on_heartbeat(&mut self) -> Result<Running, Error> {
        match &self.connection.local_state() {
            ConnectionState::Start | ConnectionState::CloseSent => return Ok(Running::Continue),
            ConnectionState::End => return Ok(Running::Stop),
            _ => {}
        }

        let frame = Frame::empty();
        self.transport.send(frame).await?;
        Ok(Running::Continue)
    }

    async fn event_loop(mut self) -> Result<(), Error> {
        loop {
            let result = tokio::select! {
                _ = self.heartbeat.next() => self.on_heartbeat().await,
                incoming = self.transport.next() => {
                    println!(">>> Debug: connection incoming frames");
                    match incoming {
                        Some(incoming) => self.on_incoming(incoming.map_err(Into::into)).await,
                        None => {
                            // Incoming stream is closed
                            println!(">>> Debug: Incoming connection is dropped");
                            Ok(Running::Stop)
                        },
                    }
                },
                control = self.control.recv() => {
                    println!(">>> Debug: connection control");
                    match control {
                        Some(control) => self.on_control(control).await,
                        None => {
                            // All control channel are dropped (which is impossible)
                            Ok(Running::Stop)
                        }
                    }
                },
                frame = self.outgoing_session_frames.recv() => {
                    println!(">>> Debug: connection outgoing session frames");
                    match frame {
                        Some(frame) => self.on_outgoing_session_frames(frame).await,
                        None => {
                            // all sessions are dropped
                            Ok(Running::Stop)
                        }
                    }
                }
            };

            match result {
                Ok(running) => match running {
                    Running::Continue => {}
                    Running::Stop => break,
                },
                Err(err) => {
                    // TODO: error handling
                    panic!("{:?}", err)
                }
            }
        }

        println!(">>> Debug: ConnectionEngine exiting event_loop");

        Ok(())
    }
}