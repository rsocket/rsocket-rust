use super::client::TcpClientTransport;
use rsocket_rust::transport::{ClientTransport, ServerTransport};
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
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
        starter: Option<Box<dyn FnMut() + Send + Sync>>,
        acceptor: impl Fn(Self::Item) + Send + Sync + 'static,
    ) -> Pin<Box<dyn Send + Future<Output = Result<(), Box<dyn Send + Sync + Error>>>>>
    where
        Self::Item: ClientTransport + Sized,
    {
        Box::pin(async move {
            match TcpListener::bind(&self.addr).await {
                Ok(mut listener) => {
                    debug!("listening on: {}", &self.addr);
                    if let Some(mut bingo) = starter {
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
