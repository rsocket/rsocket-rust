use super::client::WebsocketClientTransport;
use rsocket_rust::transport::{BoxResult, SafeFuture, ServerTransport};
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub struct WebsocketServerTransport {
    addr: SocketAddr,
}

impl From<SocketAddr> for WebsocketServerTransport {
    fn from(addr: SocketAddr) -> WebsocketServerTransport {
        WebsocketServerTransport { addr }
    }
}

impl From<String> for WebsocketServerTransport {
    fn from(addr: String) -> WebsocketServerTransport {
        let socket_addr = addr.parse().unwrap();
        WebsocketServerTransport { addr: socket_addr }
    }
}

impl From<&str> for WebsocketServerTransport {
    fn from(addr: &str) -> WebsocketServerTransport {
        let socket_addr = addr.parse().unwrap();
        WebsocketServerTransport { addr: socket_addr }
    }
}

impl ServerTransport for WebsocketServerTransport {
    type Item = WebsocketClientTransport;

    fn start(
        self,
        starter: Option<fn()>,
        acceptor: impl Fn(WebsocketClientTransport) + Send + Sync + 'static,
    ) -> SafeFuture<BoxResult<()>> {
        Box::pin(async move {
            match TcpListener::bind(self.addr).await {
                Ok(mut listener) => {
                    if let Some(bingo) = starter {
                        bingo();
                    }
                    while let Ok((socket, _)) = listener.accept().await {
                        let tp = WebsocketClientTransport::from(socket);
                        acceptor(tp);
                    }
                    Ok(())
                }
                Err(e) => Err(e.into_inner().unwrap()),
            }
        })
    }
}
