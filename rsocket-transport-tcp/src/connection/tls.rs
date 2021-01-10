use super::codec::LengthBasedFrameCodec;
use futures::{SinkExt, StreamExt};
use rsocket_rust::error::RSocketError;
use rsocket_rust::transport::{Connection, FrameSink, FrameStream};
use tokio::net::TcpStream;
use tokio_native_tls::TlsStream;
use tokio_util::codec::Framed;

#[derive(Debug)]
pub struct TlsConnection {
    stream: TlsStream<TcpStream>,
}

impl Connection for TlsConnection {
    fn split(self) -> (Box<FrameSink>, Box<FrameStream>) {
        let (sink, stream) = Framed::new(self.stream, LengthBasedFrameCodec).split();
        (
            Box::new(sink.sink_map_err(|e| RSocketError::Other(e.into()))),
            Box::new(stream.map(|it| it.map_err(|e| RSocketError::Other(e.into())))),
        )
    }
}

impl From<TlsStream<TcpStream>> for TlsConnection {
    fn from(stream: TlsStream<TcpStream>) -> Self {
        Self { stream }
    }
}
