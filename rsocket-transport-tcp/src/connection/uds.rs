use super::codec::LengthBasedFrameCodec;
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use rsocket_rust::async_trait;
use rsocket_rust::frame::Frame;
use rsocket_rust::transport::{Connection, Reader, Writer};
use rsocket_rust::{error::RSocketError, Result};
use tokio::net::UnixStream;
use tokio_util::codec::Framed;

#[derive(Debug)]
pub struct UnixConnection {
    stream: UnixStream,
}

struct InnerWriter {
    sink: SplitSink<Framed<UnixStream, LengthBasedFrameCodec>, Frame>,
}

struct InnerReader {
    stream: SplitStream<Framed<UnixStream, LengthBasedFrameCodec>>,
}

impl Connection for UnixConnection {
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
            .map(|it| it.map_err(|e| RSocketError::IO(e).into()))
    }
}

impl From<UnixStream> for UnixConnection {
    fn from(stream: UnixStream) -> UnixConnection {
        UnixConnection { stream }
    }
}
