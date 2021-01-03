use crate::client::TlsClientTransport;
use async_trait::async_trait;
use rsocket_rust::{error::RSocketError, transport::ServerTransport, Result};
use std::net::SocketAddr;
use tokio::net::TcpListener;
use tokio_native_tls::TlsAcceptor;

pub struct TlsServerTransport {
    addr: SocketAddr,
    listener: Option<TcpListener>,
    tls_acceptor: TlsAcceptor,
}

impl TlsServerTransport {
    pub fn new(addr: SocketAddr, tls_acceptor: TlsAcceptor) -> Self {
        Self {
            addr,
            listener: None,
            tls_acceptor,
        }
    }
}

#[async_trait]
impl ServerTransport for TlsServerTransport {
    type Item = TlsClientTransport;

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
                Ok((socket, _)) => {
                    let tls_acceptor = self.tls_acceptor.clone();
                    match tls_acceptor.accept(socket).await {
                        Ok(stream) => Some(Ok(TlsClientTransport::from(stream))),
                        Err(e) => Some(Err(RSocketError::Other(e.into()).into())),
                    }
                }
                Err(e) => Some(Err(RSocketError::IO(e).into())),
            },
            None => None,
        }
    }
}
