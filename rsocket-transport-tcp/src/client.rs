use super::codec::LengthBasedFrameCodec;
use futures::{SinkExt, StreamExt};
use rsocket_rust::error::RSocketError;
use rsocket_rust::frame::Frame;
use rsocket_rust::runtime::{DefaultSpawner, Spawner};
use rsocket_rust::transport::{ClientTransport, Rx, Tx, TxOnce};
use std::future::Future;
use std::net::SocketAddr;
use std::net::TcpStream as StdTcpStream;
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

enum Connector {
    Direct(TcpStream),
    Lazy(SocketAddr),
}

pub struct TcpClientTransport {
    connector: Connector,
}

impl TcpClientTransport {
    #[inline]
    fn new(connector: Connector) -> TcpClientTransport {
        TcpClientTransport { connector }
    }

    #[inline]
    async fn connect(self) -> Result<TcpStream, RSocketError> {
        match self.connector {
            Connector::Direct(stream) => Ok(stream),
            Connector::Lazy(addr) => match StdTcpStream::connect(&addr) {
                Ok(raw) => match TcpStream::from_std(raw) {
                    Ok(stream) => Ok(stream),
                    Err(e) => Err(RSocketError::from(e)),
                },
                Err(e) => Err(RSocketError::from(e)),
            },
        }
    }
}

impl ClientTransport for TcpClientTransport {
    fn attach(
        self,
        incoming: Tx<Frame>,
        mut sending: Rx<Frame>,
        connected: Option<TxOnce<Result<(), RSocketError>>>,
    ) {
        DefaultSpawner.spawn(async move {
            match self.connect().await {
                Ok(socket) => {
                    if let Some(sender) = connected {
                        sender.send(Ok(())).unwrap();
                    }
                    let (mut writer, mut reader) =
                        Framed::new(socket, LengthBasedFrameCodec).split();
                    DefaultSpawner.spawn(async move {
                        while let Some(it) = reader.next().await {
                            incoming.unbounded_send(it.unwrap()).unwrap();
                        }
                    });
                    // loop write
                    while let Some(it) = sending.next().await {
                        debug!("===> SND: {:?}", &it);
                        writer.send(it).await.unwrap()
                    }
                }
                Err(e) => {
                    if let Some(sender) = connected {
                        sender.send(Err(e)).unwrap();
                    }
                }
            }
        });
    }
}

impl From<SocketAddr> for TcpClientTransport {
    fn from(addr: SocketAddr) -> TcpClientTransport {
        TcpClientTransport::new(Connector::Lazy(addr))
    }
}

impl From<&str> for TcpClientTransport {
    fn from(addr: &str) -> TcpClientTransport {
        let socket_addr: SocketAddr = if addr.starts_with("tcp://") {
            let ss: String = addr.chars().skip(6).collect();
            ss.parse().unwrap()
        } else {
            addr.parse().unwrap()
        };
        TcpClientTransport::new(Connector::Lazy(socket_addr))
    }
}

impl From<TcpStream> for TcpClientTransport {
    fn from(socket: TcpStream) -> TcpClientTransport {
        TcpClientTransport::new(Connector::Direct(socket))
    }
}
