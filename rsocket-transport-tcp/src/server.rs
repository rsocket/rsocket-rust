use super::client::TcpClientTransport;
use rsocket_rust::transport::{BoxResult, ClientTransport, SafeFuture, ServerTransport};
use std::net::SocketAddr;
use tokio::net::TcpListener;

pub struct TcpServerTransport {
    addr: SocketAddr,
}

impl TcpServerTransport {
    fn new(addr: SocketAddr) -> TcpServerTransport {
        TcpServerTransport { addr }
    }
}

impl ServerTransport for TcpServerTransport {
    type Item = TcpClientTransport;

    fn start(
        self,
        starter: Option<fn()>,
        acceptor: impl Fn(Self::Item) + Send + Sync + 'static,
    ) -> SafeFuture<BoxResult<()>>
    where
        Self::Item: ClientTransport + Sized,
    {
        Box::pin(async move {
            match TcpListener::bind(&self.addr).await {
                Ok(mut listener) => {
                    debug!("listening on: {}", &self.addr);
                    if let Some(bingo) = starter {
                        bingo();
                    }
                    while let Ok((socket, _)) = listener.accept().await {
                        let tp = TcpClientTransport::from(socket);
                        acceptor(tp);
                    }
                    Ok(())
                }
                Err(e) => Err(e.into_inner().unwrap()),
            }
        })
    }
}

impl From<SocketAddr> for TcpServerTransport {
    fn from(addr: SocketAddr) -> TcpServerTransport {
        TcpServerTransport::new(addr)
    }
}

impl From<String> for TcpServerTransport {
    fn from(addr: String) -> TcpServerTransport {
        TcpServerTransport::new(addr.parse().unwrap())
    }
}

impl From<&str> for TcpServerTransport {
    fn from(addr: &str) -> TcpServerTransport {
        TcpServerTransport::new(addr.parse().unwrap())
    }
}
