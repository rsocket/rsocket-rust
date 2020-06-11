use super::codec::LengthBasedFrameCodec;
use futures::{SinkExt, StreamExt};
use std::os::unix::net::UnixStream as StdUnixStream;
use std::str::FromStr;
use std::io;
use tokio::net::UnixStream;
use tokio_util::codec::Framed;
use rsocket_rust::error::RSocketError;
use rsocket_rust::frame::Frame;
use rsocket_rust::runtime::{DefaultSpawner, Spawner};
use rsocket_rust::transport::{ClientTransport, Rx, Tx, TxOnce};

enum Connector {
    Direct(UnixStream),
    Lazy(String),
}

pub struct UnixClientTransport {
    connector: Connector,
}

impl UnixClientTransport {

    #[inline]
    fn new(connector: Connector) -> UnixClientTransport {
        UnixClientTransport{ connector }
    }

    #[inline]
    async fn connect(self) -> Result<UnixStream, RSocketError> {
        match self.connector {
            Connector::Direct(stream) => Ok(stream),
            Connector::Lazy(addr) => match StdUnixStream::connect(&addr) {
                Ok(raw) => match UnixStream::from_std(raw) {
                    Ok(stream) => Ok(stream),
                    Err(e) => Err(RSocketError::from(e)),
                },
                Err(e) => Err(RSocketError::from(e)),
            },
        }
    }
}

impl ClientTransport for UnixClientTransport {
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

impl FromStr for UnixClientTransport {
    type Err = io::Error;

    fn from_str(addr: &str) -> Result<Self, Self::Err> {
        let socket_addr = if addr.starts_with("unix://") || addr.starts_with("UNIX://") {
            addr.chars().skip(7).collect::<String>()
        } else {
            addr.to_string()
        };
        Ok(UnixClientTransport::new(Connector::Lazy(socket_addr)))
    }
}

impl From<String> for UnixClientTransport {
    fn from(addr: String) -> UnixClientTransport {
        UnixClientTransport::new(Connector::Lazy(addr))
    }
}

impl From<&str> for UnixClientTransport {
    fn from(addr: &str) -> UnixClientTransport {
        let socket_addr: String = if addr.starts_with("tcp://") || addr.starts_with("TCP://") {
            addr.chars().skip(6).collect::<String>()
        } else {
            addr.to_string()
        };
        UnixClientTransport::new(Connector::Lazy(socket_addr))
    }
}

impl From<UnixStream> for UnixClientTransport {
    fn from(socket: UnixStream) -> UnixClientTransport {
        UnixClientTransport::new(Connector::Direct(socket))
    }
}
