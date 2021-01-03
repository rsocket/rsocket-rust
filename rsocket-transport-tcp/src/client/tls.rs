use crate::connection::TlsConnection;
use async_trait::async_trait;
use rsocket_rust::{error::RSocketError, transport::Transport, Result};
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_native_tls::{TlsConnector, TlsStream};

#[derive(Debug)]
enum Connector {
    Direct(TlsStream<TcpStream>),
    Lazy(String, SocketAddr, TlsConnector),
}

pub struct TlsClientTransport {
    connector: Connector,
}

impl TlsClientTransport {
    pub fn new(domain: String, addr: SocketAddr, connector: TlsConnector) -> Self {
        Self {
            connector: Connector::Lazy(domain, addr, connector),
        }
    }
}

#[async_trait]
impl Transport for TlsClientTransport {
    type Conn = TlsConnection;

    async fn connect(self) -> Result<Self::Conn> {
        match self.connector {
            Connector::Direct(stream) => Ok(TlsConnection::from(stream)),
            Connector::Lazy(domain, addr, cx) => match TcpStream::connect(addr).await {
                Ok(stream) => match cx.connect(&domain, stream).await {
                    Ok(stream) => Ok(TlsConnection::from(stream)),
                    Err(e) => Err(RSocketError::Other(e.into()).into()),
                },
                Err(e) => Err(RSocketError::IO(e).into()),
            },
        }
    }
}

impl From<TlsStream<TcpStream>> for TlsClientTransport {
    fn from(stream: TlsStream<TcpStream>) -> Self {
        Self {
            connector: Connector::Direct(stream),
        }
    }
}
