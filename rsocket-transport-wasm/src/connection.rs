use async_trait::async_trait;
use futures_channel::mpsc;
use futures_util::{SinkExt, StreamExt};
use rsocket_rust::transport::{Connection, Reader, Writer};
use rsocket_rust::{error::RSocketError, frame::Frame, Result};

#[derive(Debug)]
pub struct WebsocketConnection {
    rx: mpsc::Receiver<Frame>,
    tx: mpsc::Sender<Frame>,
}

struct InnerWriter {
    tx: mpsc::Sender<Frame>,
}

struct InnerReader {
    rx: mpsc::Receiver<Frame>,
}

impl WebsocketConnection {
    pub(crate) fn new(tx: mpsc::Sender<Frame>, rx: mpsc::Receiver<Frame>) -> WebsocketConnection {
        WebsocketConnection { rx, tx }
    }
}

#[async_trait]
impl Writer for InnerWriter {
    async fn write(&mut self, frame: Frame) -> Result<()> {
        match self.tx.send(frame).await {
            Ok(()) => Ok(()),
            Err(e) => Err(RSocketError::Other(e.into()).into()),
        }
    }
}

#[async_trait]
impl Reader for InnerReader {
    async fn read(&mut self) -> Option<Result<Frame>> {
        match self.rx.next().await {
            Some(frame) => Some(Ok(frame)),
            None => None,
        }
    }
}

impl Connection for WebsocketConnection {
    fn split(
        self,
    ) -> (
        Box<dyn Writer + Send + Unpin>,
        Box<dyn Reader + Send + Unpin>,
    ) {
        (
            Box::new(InnerWriter { tx: self.tx }),
            Box::new(InnerReader { rx: self.rx }),
        )
    }
}
