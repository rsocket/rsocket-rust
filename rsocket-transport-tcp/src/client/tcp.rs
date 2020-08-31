use crate::{connection::TcpConnection, misc::parse_tcp_addr};
use async_trait::async_trait;
use rsocket_rust::{transport::Transport, Result};
use std::net::SocketAddr;
use tokio::net::TcpStream;

#[derive(Debug)]
enum Connector {
    Direct(TcpStream),
    Lazy(SocketAddr),
}

#[derive(Debug)]
pub struct TcpClientTransport {
    connector: Connector,
}

#[async_trait]
impl Transport for TcpClientTransport {
    type Conn = TcpConnection;

    async fn connect(self) -> Result<TcpConnection> {
        match self.connector {
            Connector::Direct(socket) => Ok(TcpConnection::from(socket)),
            Connector::Lazy(addr) => match TcpStream::connect(addr).await {
                Ok(stream) => Ok(TcpConnection::from(stream)),
                Err(e) => Err(Box::new(e)),
            },
        }
    }
}

impl From<TcpStream> for TcpClientTransport {
    fn from(socket: TcpStream) -> TcpClientTransport {
        TcpClientTransport {
            connector: Connector::Direct(socket),
        }
    }
}

impl From<SocketAddr> for TcpClientTransport {
    fn from(addr: SocketAddr) -> TcpClientTransport {
        TcpClientTransport {
            connector: Connector::Lazy(addr),
        }
    }
}

impl From<String> for TcpClientTransport {
    fn from(addr: String) -> Self {
        let socket_addr: SocketAddr = parse_tcp_addr(addr)
            .parse()
            .expect("Invalid transport string!");
        TcpClientTransport {
            connector: Connector::Lazy(socket_addr),
        }
    }
}

impl From<&str> for TcpClientTransport {
    fn from(addr: &str) -> TcpClientTransport {
        let socket_addr: SocketAddr = parse_tcp_addr(addr)
            .parse()
            .expect("Invalid transport string!");
        TcpClientTransport {
            connector: Connector::Lazy(socket_addr),
        }
    }
}
