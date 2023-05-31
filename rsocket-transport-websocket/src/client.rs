use std::net::SocketAddr;

use rsocket_rust::{async_trait, error::RSocketError, transport::Transport, Result};
use tokio::net::TcpStream;
use tokio_tungstenite::{
    accept_async, connect_async, tungstenite::handshake::client::Request, MaybeTlsStream,
};
use url::Url;

use super::connection::WebsocketConnection;

pub type WebsocketRequest = Request;

const WS_PROTO: &str = "ws://";
const WSS_PROTO: &str = "wss://";

#[derive(Debug)]
pub(crate) enum Connector {
    Direct(MaybeTlsStream<TcpStream>),
    Url(Url),
    Request(WebsocketRequest),
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
            Connector::Url(u) => match connect_async(u).await {
                Ok((stream, _)) => Ok(WebsocketConnection::new(stream)),
                Err(e) => Err(RSocketError::Other(e.into()).into()),
            },
            Connector::Request(req) => match connect_async(req).await {
                Ok((stream, _)) => Ok(WebsocketConnection::new(stream)),
                Err(e) => Err(RSocketError::Other(e.into()).into()),
            },
        }
    }
}

impl From<MaybeTlsStream<TcpStream>> for WebsocketClientTransport {
    fn from(socket: MaybeTlsStream<TcpStream>) -> WebsocketClientTransport {
        WebsocketClientTransport::new(Connector::Direct(socket))
    }
}

impl From<TcpStream> for WebsocketClientTransport {
    fn from(socket: TcpStream) -> WebsocketClientTransport {
        WebsocketClientTransport::new(Connector::Direct(MaybeTlsStream::Plain(socket)))
    }
}

impl From<&str> for WebsocketClientTransport {
    fn from(addr: &str) -> WebsocketClientTransport {
        let u = if addr.starts_with(WS_PROTO) || addr.starts_with(WSS_PROTO) {
            Url::parse(addr).unwrap()
        } else {
            Url::parse(&format!("{}{}", WS_PROTO, addr)).unwrap()
        };
        WebsocketClientTransport::new(Connector::Url(u))
    }
}

impl From<SocketAddr> for WebsocketClientTransport {
    fn from(addr: SocketAddr) -> WebsocketClientTransport {
        let u = Url::parse(&format!("{}{}", WS_PROTO, addr)).unwrap();
        WebsocketClientTransport::new(Connector::Url(u))
    }
}

impl From<Url> for WebsocketClientTransport {
    fn from(url: Url) -> WebsocketClientTransport {
        WebsocketClientTransport::new(Connector::Url(url))
    }
}

impl From<Request> for WebsocketClientTransport {
    fn from(req: WebsocketRequest) -> Self {
        WebsocketClientTransport::new(Connector::Request(req))
    }
}
