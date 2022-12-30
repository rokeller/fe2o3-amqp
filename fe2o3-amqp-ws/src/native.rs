use futures_util::{Stream, Sink};
use pin_project_lite::pin_project;
use tokio::{
    io::{AsyncRead, AsyncWrite},
    net::TcpStream,
};
use tokio_tungstenite::{
    client_async, client_async_with_config, connect_async, connect_async_with_config,
    MaybeTlsStream,
};

use tungstenite::{
    client::IntoClientRequest,
    handshake::client::{Request, Response},
    http::HeaderValue,
    protocol::WebSocketConfig,
};

use crate::WsMessage;

use super::{Error, WebSocketStream};

const SEC_WEBSOCKET_PROTOCOL: &str = "Sec-WebSocket-Protocol";

// type TokioWebSocketStream<S> = tokio_tungstenite::WebSocketStream<MaybeTlsStream<S>>;

pin_project! {
    /// This a simple wrapper around [`tokio_tungstenite::WebSocketStream`]
    #[derive(Debug)]
    pub struct TokioWebSocketStream<S>{
        #[pin]
        stream: tokio_tungstenite::WebSocketStream<S>
    }
}

impl<S> From<tokio_tungstenite::WebSocketStream<S>> for WebSocketStream<TokioWebSocketStream<S>> {
    fn from(inner: tokio_tungstenite::WebSocketStream<S>) -> Self {
        Self {
            inner: TokioWebSocketStream { stream: inner },
            current_binary: None,
        }
    }
}

impl<S> Stream for TokioWebSocketStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    type Item = Result<WsMessage, tungstenite::Error>;

    fn poll_next(
        self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Option<Self::Item>> {
        let this = self.project();
        this.stream
            .poll_next(cx)
            .map(|item| item.map(|item| item.map(|msg| WsMessage(msg))))
    }
}

impl<S> Sink<WsMessage> for TokioWebSocketStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    type Error = tungstenite::Error;

    fn poll_ready(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.stream.poll_ready(cx)
    }

    fn start_send(self: std::pin::Pin<&mut Self>, item: WsMessage) -> Result<(), Self::Error> {
        let this = self.project();
        this.stream.start_send(item.0)
    }

    fn poll_flush(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.stream.poll_flush(cx)
    }

    fn poll_close(self: std::pin::Pin<&mut Self>, cx: &mut std::task::Context<'_>) -> std::task::Poll<Result<(), Self::Error>> {
        let this = self.project();
        this.stream.poll_close(cx)
    }
}

impl WebSocketStream<TokioWebSocketStream<MaybeTlsStream<TcpStream>>> {
    /// Calls [`tokio_tungstenite::connect_async`] internally with `"Sec-WebSocket-Protocol"` HTTP
    /// header of the `req` set to `"amqp"`
    pub async fn connect(req: impl IntoClientRequest) -> Result<(Self, Response), Error> {
        let request = map_amqp_websocket_request(req)?;
        let (mut ws_stream, response) = connect_async(request).await?;
        match verify_response(response) {
            Ok(response) => Ok((Self::from(ws_stream), response)),
            Err(error) => {
                ws_stream.close(None).await?;
                Err(error)
            }
        }
    }

    /// Calls [`tokio_tungstenite::connect_async_with_config`] internally with
    /// `"Sec-WebSocket-Protocol"` HTTP header of the `req` set to `"amqp"`
    pub async fn connect_with_config(
        req: impl IntoClientRequest,
        config: Option<WebSocketConfig>,
    ) -> Result<(Self, Response), Error> {
        let request = map_amqp_websocket_request(req)?;
        let (mut ws_stream, response) = connect_async_with_config(request, config).await?;
        match verify_response(response) {
            Ok(response) => Ok((Self::from(ws_stream), response)),
            Err(error) => {
                ws_stream.close(None).await?;
                Err(error)
            }
        }
    }
}

impl<S> WebSocketStream<TokioWebSocketStream<S>>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Calls [`tokio_tungstenite::client_async`] internally with `"Sec-WebSocket-Protocol"` HTTP
    /// header of the `req` set to `"amqp"`
    pub async fn connect_with_stream(
        req: impl IntoClientRequest,
        stream: S,
    ) -> Result<(Self, Response), Error> {
        let request = map_amqp_websocket_request(req)?;
        let (mut ws_stream, response) = client_async(request, stream).await?;
        match verify_response(response) {
            Ok(response) => Ok((Self::from(ws_stream), response)),
            Err(error) => {
                ws_stream.close(None).await?;
                Err(error)
            }
        }
    }

    /// Calls [`tokio_tungstenite::client_async_with_config`] internally with
    /// `"Sec-WebSocket-Protocol"` HTTP header of the `req` set to `"amqp"`
    pub async fn connect_with_stream_and_config(
        req: impl IntoClientRequest,
        stream: S,
        config: Option<WebSocketConfig>,
    ) -> Result<(Self, Response), Error> {
        let request = map_amqp_websocket_request(req)?;
        let (mut ws_stream, response) = client_async_with_config(request, stream, config).await?;
        match verify_response(response) {
            Ok(response) => Ok((Self::from(ws_stream), response)),
            Err(error) => {
                ws_stream.close(None).await?;
                Err(error)
            }
        }
    }
}

#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "native-tls",
        feature = "native-tls-vendored",
        feature = "rustls-tls-native-roots",
        feature = "rustls-tls-webpki-roots"
    )))
)]
#[cfg(any(
    feature = "native-tls",
    feature = "native-tls-vendored",
    feature = "rustls-tls-native-roots",
    feature = "rustls-tls-webpki-roots"
))]
impl<S> WebSocketStream<TokioWebSocketStream<MaybeTlsStream<S>>>
where
    S: AsyncRead + AsyncWrite + Send + Sync + Unpin + 'static,
{
    /// Calls [`tokio_tungstenite::client_async_tls`] internally with `"Sec-WebSocket-Protocol"` HTTP
    /// header of the `req` set to `"amqp"`
    pub async fn connect_tls_with_stream(
        req: impl IntoClientRequest,
        stream: S,
    ) -> Result<(Self, Response), Error> {
        let request = map_amqp_websocket_request(req)?;
        let (mut ws_stream, response) =
            tokio_tungstenite::client_async_tls(request, stream).await?;
        match verify_response(response) {
            Ok(response) => Ok((Self::from(ws_stream), response)),
            Err(error) => {
                ws_stream.close(None).await?;
                Err(error)
            }
        }
    }

    /// Calls [`tokio_tungstenite::client_async_tls_with_config`] internally with
    /// `"Sec-WebSocket-Protocol"` HTTP header of the `req` set to `"amqp"`
    pub async fn connect_tls_with_stream_and_config(
        req: impl IntoClientRequest,
        stream: S,
        config: Option<WebSocketConfig>,
        connector: Option<tokio_tungstenite::Connector>,
    ) -> Result<(Self, Response), Error> {
        let request = map_amqp_websocket_request(req)?;
        let (mut ws_stream, response) =
            tokio_tungstenite::client_async_tls_with_config(request, stream, config, connector)
                .await?;
        match verify_response(response) {
            Ok(response) => Ok((Self::from(ws_stream), response)),
            Err(error) => {
                ws_stream.close(None).await?;
                Err(error)
            }
        }
    }
}

#[cfg_attr(
    docsrs,
    doc(cfg(any(
        feature = "native-tls",
        feature = "native-tls-vendored",
        feature = "rustls-tls-native-roots",
        feature = "rustls-tls-webpki-roots"
    )))
)]
#[cfg(any(
    feature = "native-tls",
    feature = "native-tls-vendored",
    feature = "rustls-tls-native-roots",
    feature = "rustls-tls-webpki-roots"
))]
impl WebSocketStream<TokioWebSocketStream<MaybeTlsStream<TcpStream>>> {
    /// Calls [`tokio_tungstenite::connect_async_tls_with_config`] internally with
    /// `"Sec-WebSocket-Protocol"` HTTP header of the `req` set to `"amqp"`
    pub async fn connect_tls_with_config(
        req: impl IntoClientRequest,
        config: Option<WebSocketConfig>,
        connector: Option<tokio_tungstenite::Connector>,
    ) -> Result<(Self, Response), Error> {
        let request = map_amqp_websocket_request(req)?;
        let (mut ws_stream, response) =
            tokio_tungstenite::connect_async_tls_with_config(request, config, connector).await?;
        match verify_response(response) {
            Ok(response) => Ok((Self::from(ws_stream), response)),
            Err(error) => {
                ws_stream.close(None).await?;
                Err(error)
            }
        }
    }
}

fn map_amqp_websocket_request(req: impl IntoClientRequest) -> Result<Request, tungstenite::Error> {
    let mut request = req.into_client_request()?;

    // Sec-WebSocket-Protocol HTTP header
    //
    // Identifies the WebSocket subprotocol. For this AMQP WebSocket binding, the value MUST be
    // set to the US- ASCII text string “amqp” which refers to the 1.0 version of the AMQP 1.0
    // or greater, with version negotiation as defined by AMQP 1.0.
    request
        .headers_mut()
        .insert(SEC_WEBSOCKET_PROTOCOL, HeaderValue::from_static("amqp"));

    Ok(request)
}

fn verify_response(response: Response) -> Result<Response, Error> {
    use http::StatusCode;

    // If the Client does not receive a response with HTTP status code 101 and an HTTP
    // Sec-WebSocket-Protocol equal to the US-ASCII text string “amqp” then the Client MUST close
    // the socket connection
    if response.status() != StatusCode::SWITCHING_PROTOCOLS {
        return Err(Error::StatucCodeIsNotSwitchingProtocols);
    }

    match response
        .headers()
        .get(SEC_WEBSOCKET_PROTOCOL)
        .map(|val| val.to_str())
        .ok_or(Error::MissingSecWebSocketProtocol)??
    {
        "amqp" => Ok(response),
        _ => Err(Error::SecWebSocketProtocolIsNotAmqp),
    }
}
