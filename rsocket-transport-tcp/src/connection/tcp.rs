use futures::{SinkExt, StreamExt};
use rsocket_rust::error::RSocketError;
use rsocket_rust::transport::{Connection, FrameSink, FrameStream};
use tokio::net::TcpStream;
use tokio_util::codec::Framed;

use super::codec::LengthBasedFrameCodec;

#[derive(Debug)]
pub struct TcpConnection {
    stream: TcpStream,
}

impl Connection for TcpConnection {
    fn split(self) -> (Box<FrameSink>, Box<FrameStream>) {
        let (sink, stream) = Framed::new(self.stream, LengthBasedFrameCodec).split();
        (
            Box::new(sink.sink_map_err(|e| RSocketError::Other(e.into()))),
            Box::new(stream.map(|next| next.map_err(|e| RSocketError::Other(e.into())))),
        )
    }
}

impl From<TcpStream> for TcpConnection {
    fn from(stream: TcpStream) -> TcpConnection {
        TcpConnection { stream }
    }
}
