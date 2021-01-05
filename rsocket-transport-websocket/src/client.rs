use super::connection::WebsocketConnection;
use rsocket_rust::{async_trait, error::RSocketError, transport::Transport, Result};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, connect_async};
use url::Url;

#[derive(Debug)]
pub(crate) enum Connector {
    Direct(TcpStream),
    Lazy(Url),
}

#[derive(Debug)]
pub struct WebsocketClientTransport {
    connector: Connector,
}

impl WebsocketClientTransport {
    pub(crate) fn new(connector: Connector) -> WebsocketClientTransport {
        WebsocketClientTransport { connector }
    }
}

#[async_trait]
impl Transport for WebsocketClientTransport {
    type Conn = WebsocketConnection;

    async fn connect(self) -> Result<WebsocketConnection> {
        match self.connector {
            Connector::Direct(stream) => match accept_async(stream).await {
                Ok(ws) => Ok(WebsocketConnection::new(ws)),
                Err(e) => Err(RSocketError::Other(e.into()).into()),
            },
            Connector::Lazy(u) => match connect_async(u).await {
                Ok((stream, _)) => Ok(WebsocketConnection::new(stream)),
                Err(e) => Err(RSocketError::Other(e.into()).into()),
            },
        }
    }
}

impl From<TcpStream> for WebsocketClientTransport {
    fn from(socket: TcpStream) -> WebsocketClientTransport {
        WebsocketClientTransport::new(Connector::Direct(socket))
    }
}

impl From<&str> for WebsocketClientTransport {
    fn from(addr: &str) -> WebsocketClientTransport {
        let u = if addr.starts_with("ws://") {
            Url::parse(addr).unwrap()
        } else {
            Url::parse(&format!("ws://{}", addr)).unwrap()
        };
        WebsocketClientTransport::new(Connector::Lazy(u))
    }
}

impl From<SocketAddr> for WebsocketClientTransport {
    fn from(addr: SocketAddr) -> WebsocketClientTransport {
        let u = Url::parse(&format!("ws://{}", addr)).unwrap();
        WebsocketClientTransport::new(Connector::Lazy(u))
    }
}

impl From<Url> for WebsocketClientTransport {
    fn from(url: Url) -> WebsocketClientTransport {
        WebsocketClientTransport::new(Connector::Lazy(url))
    }
}
