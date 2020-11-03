use super::client::WebsocketClientTransport;
use async_trait::async_trait;
use rsocket_rust::{error::RSocketError, transport::ServerTransport, Result};
use std::net::SocketAddr;
use tokio::net::TcpListener;

const WS_PROTO: &str = "ws://";

#[derive(Debug)]
pub struct WebsocketServerTransport {
    addr: SocketAddr,
    listener: Option<TcpListener>,
}

#[async_trait]
impl ServerTransport for WebsocketServerTransport {
    type Item = WebsocketClientTransport;

    async fn start(&mut self) -> Result<()> {
        if self.listener.is_some() {
            warn!("websocket server transport started already!");
            return Ok(());
        }
        match TcpListener::bind(self.addr).await {
            Ok(listener) => {
                self.listener = Some(listener);
                Ok(())
            }
            Err(e) => Err(RSocketError::IO(e).into()),
        }
    }

    async fn next(&mut self) -> Option<Result<WebsocketClientTransport>> {
        match self.listener.as_mut() {
            Some(listener) => match listener.accept().await {
                Ok((socket, _)) => Some(Ok(WebsocketClientTransport::from(socket))),
                Err(e) => Some(Err(RSocketError::Other(e.into()).into())),
            },
            None => None,
        }
    }
}

#[inline]
fn parse_socket_addr(addr: impl AsRef<str>) -> SocketAddr {
    let addr = addr.as_ref();
    if addr.starts_with(WS_PROTO) {
        addr.chars()
            .skip(WS_PROTO.len())
            .collect::<String>()
            .parse()
    } else {
        addr.parse()
    }
    .expect("Invalid transport string!")
}

impl From<SocketAddr> for WebsocketServerTransport {
    fn from(addr: SocketAddr) -> WebsocketServerTransport {
        WebsocketServerTransport {
            addr,
            listener: None,
        }
    }
}

impl From<String> for WebsocketServerTransport {
    fn from(addr: String) -> WebsocketServerTransport {
        WebsocketServerTransport {
            addr: parse_socket_addr(addr),
            listener: None,
        }
    }
}

impl From<&str> for WebsocketServerTransport {
    fn from(addr: &str) -> WebsocketServerTransport {
        WebsocketServerTransport {
            addr: parse_socket_addr(addr),
            listener: None,
        }
    }
}
