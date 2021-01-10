use super::codec::LengthBasedFrameCodec;
use futures::{SinkExt, StreamExt};
use rsocket_rust::error::RSocketError;
use rsocket_rust::transport::{Connection, FrameSink, FrameStream};
use tokio::net::UnixStream;
use tokio_util::codec::Framed;

#[derive(Debug)]
pub struct UnixConnection {
    stream: UnixStream,
}

impl Connection for UnixConnection {
    fn split(self) -> (Box<FrameSink>, Box<FrameStream>) {
        let (sink, stream) = Framed::new(self.stream, LengthBasedFrameCodec).split();
        (
            Box::new(sink.sink_map_err(|e| RSocketError::Other(e.into()))),
            Box::new(stream.map(|it| it.map_err(|e| RSocketError::Other(e.into())))),
        )
    }
}

impl From<UnixStream> for UnixConnection {
    fn from(stream: UnixStream) -> UnixConnection {
        UnixConnection { stream }
    }
}
