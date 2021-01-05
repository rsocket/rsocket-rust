use super::codec::LengthBasedFrameCodec;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use rsocket_rust::async_trait;
use rsocket_rust::frame::Frame;
use rsocket_rust::transport::{Connection, Reader, Writer};
use rsocket_rust::{error::RSocketError, Result};
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;
use tokio_util::codec::Framed;

#[derive(Debug)]
pub struct TlsConnection {
    stream: TlsStream<TcpStream>,
}

struct InnerWriter {
    sink: SplitSink<Framed<TlsStream<TcpStream>, LengthBasedFrameCodec>, Frame>,
}

struct InnerReader {
    stream: SplitStream<Framed<TlsStream<TcpStream>, LengthBasedFrameCodec>>,
}

#[async_trait]
impl Writer for InnerWriter {
    async fn write(&mut self, frame: Frame) -> Result<()> {
        match self.sink.send(frame).await {
            Ok(()) => Ok(()),
            Err(e) => Err(RSocketError::IO(e).into()),
        }
    }
}

#[async_trait]
impl Reader for InnerReader {
    async fn read(&mut self) -> Option<Result<Frame>> {
        self.stream
            .next()
            .await
            .map(|next| next.map_err(|e| RSocketError::IO(e).into()))
    }
}

impl Connection for TlsConnection {
    fn split(
        self,
    ) -> (
        Box<dyn Writer + Send + Unpin>,
        Box<dyn Reader + Send + Unpin>,
    ) {
        let (sink, stream) = Framed::new(self.stream, LengthBasedFrameCodec).split();
        (
            Box::new(InnerWriter { sink }),
            Box::new(InnerReader { stream }),
        )
    }
}

impl From<TlsStream<TcpStream>> for TlsConnection {
    fn from(stream: TlsStream<TcpStream>) -> Self {
        Self { stream }
    }
}
