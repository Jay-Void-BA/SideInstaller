//! RFC 6455 framing/handshake comes from tungstenite; rootshell supplies TLS.
use futures_util::{Sink, Stream};
use rootshell_runtime::Socket;
use std::{
    pin::Pin,
    task::{Context, Poll},
    time::Duration,
};
pub use tungstenite;
use tungstenite::{Message, WebSocket};

pub struct Connection(WebSocket<Socket>);
pub async fn connect_async(url: &str) -> Result<(Connection, ()), super::Error> {
    let parsed = url::Url::parse(url).map_err(|_| super::invalid("invalid WebSocket URL"))?;
    let tls = match parsed.scheme() {
        "wss" => true,
        "ws" => false,
        _ => return Err(super::invalid("WebSocket scheme required")),
    };
    let host = parsed
        .host_str()
        .ok_or_else(|| super::invalid("WebSocket host missing"))?;
    if !tls && !matches!(host, "127.0.0.1" | "localhost" | "[::1]") {
        return Err(super::invalid("remote WebSocket requires TLS"));
    }
    if !parsed.username().is_empty() || parsed.password().is_some() {
        return Err(super::invalid("URL credentials are not supported"));
    }
    let socket = Socket::connect(
        host.trim_matches(['[', ']']),
        parsed
            .port_or_known_default()
            .ok_or_else(|| super::invalid("WebSocket port missing"))?,
        tls,
    )?;
    socket.set_read_timeout(Duration::from_secs(30))?;
    let config = tungstenite::protocol::WebSocketConfig::default()
        .max_message_size(Some(super::MAX_BODY))
        .max_frame_size(Some(super::MAX_BODY));
    let (ws, _) = tungstenite::client::client_with_config(url, socket, Some(config))
        .map_err(|_| super::invalid("WebSocket handshake failed"))?;
    Ok((Connection(ws), ()))
}
impl Stream for Connection {
    type Item = Result<Message, tungstenite::Error>;
    fn poll_next(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Option<Self::Item>> {
        Poll::Ready(match self.get_mut().0.read() {
            Ok(message) => Some(Ok(message)),
            Err(tungstenite::Error::ConnectionClosed) => None,
            Err(e) => Some(Err(e)),
        })
    }
}
impl Sink<Message> for Connection {
    type Error = tungstenite::Error;
    fn poll_ready(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(Ok(()))
    }
    fn start_send(self: Pin<&mut Self>, item: Message) -> Result<(), Self::Error> {
        self.get_mut().0.write(item)
    }
    fn poll_flush(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(self.get_mut().0.flush())
    }
    fn poll_close(self: Pin<&mut Self>, _: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        Poll::Ready(self.get_mut().0.close(None))
    }
}
impl Connection {
    pub async fn close(
        &mut self,
        frame: Option<tungstenite::protocol::CloseFrame>,
    ) -> Result<(), tungstenite::Error> {
        self.0.close(frame)
    }
}
