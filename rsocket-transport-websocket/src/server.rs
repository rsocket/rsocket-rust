use super::client::WebsocketClientTransport;
use rsocket_rust::transport::ServerTransport;
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
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
    ) -> Pin<Box<dyn Sync + Send + Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>> {
        // TODO: see https://github.com/snapview/tokio-tungstenite/blob/master/examples/server.rs
        Box::pin(async move {
            let mut listener = TcpListener::bind(self.addr).await.unwrap();
            if let Some(bingo) = starter {
                bingo();
            }
            while let Ok((socket, _)) = listener.accept().await {
                let tp = WebsocketClientTransport::from(socket);
                acceptor(tp);
            }

            Ok(())
        })
    }
}
