use std::net::SocketAddr;

use rsocket_rust::async_trait;
use rsocket_rust::{error::RSocketError, transport::ServerTransport, Result};
use tokio::net::TcpListener;

use crate::{client::TcpClientTransport, misc::parse_tcp_addr};

#[derive(Debug)]
pub struct TcpServerTransport {
    addr: SocketAddr,
    listener: Option<TcpListener>,
}

impl TcpServerTransport {
    fn new(addr: SocketAddr) -> TcpServerTransport {
        TcpServerTransport {
            addr,
            listener: None,
        }
    }
}

#[async_trait]
impl ServerTransport for TcpServerTransport {
    type Item = TcpClientTransport;

    async fn start(&mut self) -> Result<()> {
        if self.listener.is_some() {
            return Ok(());
        }
        match TcpListener::bind(self.addr).await {
            Ok(listener) => {
                self.listener = Some(listener);
                debug!("listening on: {}", &self.addr);
                Ok(())
            }
            Err(e) => Err(RSocketError::IO(e).into()),
        }
    }

    async fn next(&mut self) -> Option<Result<Self::Item>> {
        match self.listener.as_mut() {
            Some(listener) => match listener.accept().await {
                Ok((socket, _)) => Some(Ok(TcpClientTransport::from(socket))),
                Err(e) => Some(Err(RSocketError::IO(e).into())),
            },
            None => None,
        }
    }
}

impl From<SocketAddr> for TcpServerTransport {
    fn from(addr: SocketAddr) -> TcpServerTransport {
        TcpServerTransport::new(addr)
    }
}

impl From<String> for TcpServerTransport {
    fn from(addr: String) -> TcpServerTransport {
        TcpServerTransport::new(parse_tcp_addr(addr).parse().unwrap())
    }
}

impl From<&str> for TcpServerTransport {
    fn from(addr: &str) -> TcpServerTransport {
        TcpServerTransport::new(parse_tcp_addr(addr).parse().unwrap())
    }
}
