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

pub struct TcpClientTransport {
    socket: TcpStream,
}

impl TcpClientTransport {
    #[inline]
    async fn process(
        socket: TcpStream,
        mut inputs: Rx<Frame>,
        outputs: Tx<Frame>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let (mut writer, mut reader) = Framed::new(socket, LengthBasedFrameCodec).split();
        tokio::spawn(async move {
            while let Some(it) = reader.next().await {
                outputs.send(it.unwrap()).unwrap();
            }
        });
        // loop write
        while let Some(it) = inputs.recv().await {
            debug!("===> SND: {:?}", &it);
            writer.send(it).await.unwrap()
        }
        Ok(())
    }
}

impl ClientTransport for TcpClientTransport {
    fn attach(
        self,
        incoming: Tx<Frame>,
        sending: Rx<Frame>,
    ) -> Pin<Box<dyn Sync + Send + Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>> {
        Box::pin(Self::process(self.socket, sending, incoming))
    }
}

impl From<&SocketAddr> for TcpClientTransport {
    fn from(addr: &SocketAddr) -> TcpClientTransport {
        let socket = connect(addr);
        TcpClientTransport { socket }
    }
}

impl From<SocketAddr> for TcpClientTransport {
    fn from(addr: SocketAddr) -> TcpClientTransport {
        let socket = connect(&addr);
        TcpClientTransport { socket }
    }
}

impl From<&str> for TcpClientTransport {
    fn from(addr: &str) -> TcpClientTransport {
        let socket_addr: SocketAddr = addr.parse().unwrap();
        let socket = connect(&socket_addr);
        TcpClientTransport { socket }
    }
}

impl From<TcpStream> for TcpClientTransport {
    fn from(socket: TcpStream) -> TcpClientTransport {
        TcpClientTransport { socket }
    }
}

#[inline]
fn connect(addr: &SocketAddr) -> TcpStream {
    let origin = StdTcpStream::connect(addr).unwrap();
    TcpStream::from_std(origin).unwrap()
}
