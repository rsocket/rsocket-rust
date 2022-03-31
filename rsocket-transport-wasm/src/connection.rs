use futures_channel::mpsc;
use futures_util::{SinkExt, StreamExt};
use rsocket_rust::transport::{Connection, FrameSink, FrameStream};
use rsocket_rust::{error::RSocketError, frame::Frame};

#[derive(Debug)]
pub struct WebsocketConnection {
    rx: mpsc::Receiver<Frame>,
    tx: mpsc::Sender<Frame>,
}

impl WebsocketConnection {
    pub(crate) fn new(tx: mpsc::Sender<Frame>, rx: mpsc::Receiver<Frame>) -> WebsocketConnection {
        WebsocketConnection { rx, tx }
    }
}

impl Connection for WebsocketConnection {
    fn split(self) -> (Box<FrameSink>, Box<FrameStream>) {
        (
            Box::new(self.tx.sink_map_err(|e| RSocketError::Other(e.into()))),
            Box::new(self.rx.map(Ok)),
        )
    }
}
