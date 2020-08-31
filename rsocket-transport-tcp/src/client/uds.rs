use crate::connection::UnixConnection;
use crate::misc::parse_uds_addr;
use async_trait::async_trait;
use rsocket_rust::{transport::Transport, Result};
use tokio::net::UnixStream;

#[derive(Debug)]
enum Connector {
    Direct(UnixStream),
    Lazy(String),
}

#[derive(Debug)]
pub struct UnixClientTransport {
    connector: Connector,
}

#[async_trait]
impl Transport for UnixClientTransport {
    type Conn = UnixConnection;

    async fn connect(self) -> Result<UnixConnection> {
        match self.connector {
            Connector::Direct(socket) => Ok(UnixConnection::from(socket)),
            Connector::Lazy(addr) => match UnixStream::connect(addr).await {
                Ok(stream) => Ok(UnixConnection::from(stream)),
                Err(e) => Err(Box::new(e)),
            },
        }
    }
}

impl From<UnixStream> for UnixClientTransport {
    fn from(socket: UnixStream) -> UnixClientTransport {
        UnixClientTransport {
            connector: Connector::Direct(socket),
        }
    }
}

impl From<String> for UnixClientTransport {
    fn from(addr: String) -> UnixClientTransport {
        UnixClientTransport {
            connector: Connector::Lazy(parse_uds_addr(addr)),
        }
    }
}

impl From<&str> for UnixClientTransport {
    fn from(addr: &str) -> UnixClientTransport {
        UnixClientTransport {
            connector: Connector::Lazy(parse_uds_addr(addr)),
        }
    }
}
