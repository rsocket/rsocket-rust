use async_trait::async_trait;
use bytes::{BufMut, BytesMut};
use futures::stream::{SplitSink, SplitStream};
use futures::{SinkExt, StreamExt};
use rsocket_rust::{
    error::RSocketError,
    frame::Frame,
    transport::{Connection, Reader, Writer},
    utils::Writeable,
    Result,
};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

#[derive(Debug)]
pub struct WebsocketConnection {
    stream: WebSocketStream<TcpStream>,
}

struct InnerWriter {
    sink: SplitSink<WebSocketStream<TcpStream>, Message>,
}

struct InnerReader {
    stream: SplitStream<WebSocketStream<TcpStream>>,
}

impl WebsocketConnection {
    pub(crate) fn new(stream: WebSocketStream<TcpStream>) -> WebsocketConnection {
        WebsocketConnection { stream }
    }
}

impl Connection for WebsocketConnection {
    fn split(
        self,
    ) -> (
        Box<dyn Writer + Send + Unpin>,
        Box<dyn Reader + Send + Unpin>,
    ) {
        let (sink, stream) = self.stream.split();
        (
            Box::new(InnerWriter { sink }),
            Box::new(InnerReader { stream }),
        )
    }
}

#[async_trait]
impl Writer for InnerWriter {
    async fn write(&mut self, frame: Frame) -> Result<()> {
        let mut bf = BytesMut::new();
        frame.write_to(&mut bf);
        let msg = Message::binary(bf.to_vec());
        match self.sink.send(msg).await {
            Ok(()) => Ok(()),
            Err(e) => Err(RSocketError::Other(e.into()).into()),
        }
    }
}

#[async_trait]
impl Reader for InnerReader {
    async fn read(&mut self) -> Option<Result<Frame>> {
        match self.stream.next().await {
            Some(Ok(msg)) => {
                let raw = msg.into_data();
                let mut bf = BytesMut::new();
                bf.put_slice(&raw[..]);
                match Frame::decode(&mut bf) {
                    Ok(frame) => Some(Ok(frame)),
                    Err(e) => Some(Err(e)),
                }
            }
            Some(Err(e)) => Some(Err(RSocketError::Other(e.into()).into())),
            None => None,
        }
    }
}
