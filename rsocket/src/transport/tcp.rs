use super::codec::LengthBasedFrameCodec;
use super::misc;
use super::spi::{ClientTransport, Rx, Tx};
use crate::frame::Frame;
use futures::{Sink, SinkExt, Stream, StreamExt};
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::net::TcpStream as StdTcpStream;
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc;
use tokio_util::codec::{Decoder, Encoder, Framed, FramedParts, FramedRead, FramedWrite};

pub(crate) struct TcpClientTransport {
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
            // loop read
            loop {
                match reader.next().await {
                    Some(it) => outputs.send(it.unwrap()).unwrap(),
                    None => {
                        drop(outputs);
                        break;
                    }
                }
            }
        });
        // loop write
        while let Some(it) = inputs.recv().await {
            misc::debug_frame(true, &it);
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
