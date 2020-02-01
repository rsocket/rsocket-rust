use super::codec::LengthBasedFrameCodec;
use futures::{SinkExt, StreamExt};
use rsocket_rust::frame::Frame;
use rsocket_rust::transport::{ClientTransport, Rx, Tx};
use std::error::Error;
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
    async fn connect(self) -> Result<TcpStream, Box<dyn Send + Sync + Error>> {
        match self.connector {
            Connector::Direct(stream) => Ok(stream),
            Connector::Lazy(addr) => match StdTcpStream::connect(&addr) {
                Ok(raw) => match TcpStream::from_std(raw) {
                    Ok(stream) => Ok(stream),
                    Err(e) => Err(Box::new(e)),
                },
                Err(e) => Err(Box::new(e)),
            },
        }
    }
}

impl ClientTransport for TcpClientTransport {
    fn attach(
        self,
        incoming: Tx<Frame>,
        mut sending: Rx<Frame>,
    ) -> Pin<Box<dyn Sync + Send + Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>> {
        Box::pin(async move {
            let socket = self.connect().await?;
            let (mut writer, mut reader) = Framed::new(socket, LengthBasedFrameCodec).split();
            tokio::spawn(async move {
                while let Some(it) = reader.next().await {
                    incoming.send(it.unwrap()).unwrap();
                }
            });
            // loop write
            while let Some(it) = sending.recv().await {
                debug!("===> SND: {:?}", &it);
                writer.send(it).await.unwrap()
            }
            Ok(())
        })
    }
}

impl From<SocketAddr> for TcpClientTransport {
    fn from(addr: SocketAddr) -> TcpClientTransport {
        TcpClientTransport::new(Connector::Lazy(addr))
    }
}

impl From<&str> for TcpClientTransport {
    fn from(addr: &str) -> TcpClientTransport {
        let socket_addr: SocketAddr = addr.parse().unwrap();
        TcpClientTransport::new(Connector::Lazy(socket_addr))
    }
}

impl From<TcpStream> for TcpClientTransport {
    fn from(socket: TcpStream) -> TcpClientTransport {
        TcpClientTransport::new(Connector::Direct(socket))
    }
}
