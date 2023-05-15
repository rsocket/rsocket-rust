use std::result::Result;

use bytes::{BufMut, BytesMut};
use futures::stream::SplitSink;
use futures::{Sink, SinkExt, StreamExt};
use rsocket_rust::{
    error::RSocketError,
    frame::Frame,
    transport::{Connection, FrameSink, FrameStream},
    utils::Writeable,
};
use tokio::net::TcpStream;
use tokio_tungstenite::MaybeTlsStream;
use tokio_tungstenite::{
    tungstenite::{Error as WsError, Message},
    WebSocketStream,
};

#[derive(Debug)]
pub struct WebsocketConnection {
    stream: WebSocketStream<MaybeTlsStream<TcpStream>>,
}

impl WebsocketConnection {
    pub(crate) fn new(stream: WebSocketStream<MaybeTlsStream<TcpStream>>) -> WebsocketConnection {
        WebsocketConnection { stream }
    }
}

struct InnerSink(SplitSink<WebSocketStream<MaybeTlsStream<TcpStream>>, Message>);

impl Sink<Frame> for InnerSink {
    type Error = WsError;

    fn poll_ready(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.as_mut().0.poll_ready_unpin(cx)
    }

    fn start_send(mut self: std::pin::Pin<&mut Self>, item: Frame) -> Result<(), Self::Error> {
        let mut b = BytesMut::new();
        item.write_to(&mut b);
        let msg = Message::binary(b.to_vec());
        self.as_mut().0.start_send_unpin(msg)
    }

    fn poll_flush(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.as_mut().0.poll_flush_unpin(cx)
    }

    fn poll_close(
        mut self: std::pin::Pin<&mut Self>,
        cx: &mut std::task::Context<'_>,
    ) -> std::task::Poll<Result<(), Self::Error>> {
        self.as_mut().0.poll_close_unpin(cx)
    }
}

impl Connection for WebsocketConnection {
    fn split(self) -> (Box<FrameSink>, Box<FrameStream>) {
        let (sink, stream) = self.stream.split();
        (
            Box::new(InnerSink(sink).sink_map_err(|e| RSocketError::Other(e.into()))),
            Box::new(stream.map(|it| match it {
                Ok(msg) => {
                    let raw = msg.into_data();
                    let mut bf = BytesMut::new();
                    bf.put_slice(&raw[..]);
                    match Frame::decode(&mut bf) {
                        Ok(frame) => Ok(frame),
                        Err(e) => Err(RSocketError::Other(e)),
                    }
                }
                Err(e) => Err(RSocketError::Other(e.into())),
            })),
        )
    }
}
