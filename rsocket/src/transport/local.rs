use super::spi::{BoxResult, ClientTransport, Rx, SafeFuture, Tx};
use crate::frame::Frame;
use futures::{Sink, SinkExt, Stream, StreamExt};
use std::error::Error;
use std::future::Future;
use std::pin::Pin;

pub(crate) struct LocalClientTransport {
    tx: Tx<Frame>,
    rx: Option<Rx<Frame>>,
}

impl LocalClientTransport {
    pub fn new(tx: Tx<Frame>, rx: Rx<Frame>) -> LocalClientTransport {
        LocalClientTransport { tx, rx: Some(rx) }
    }
}

impl ClientTransport for LocalClientTransport {
    fn attach(mut self, incoming: Tx<Frame>, mut sending: Rx<Frame>) -> SafeFuture<BoxResult<()>> {
        let mut rx = self.rx.take().unwrap();
        Box::pin(async move {
            tokio::spawn(async move {
                while let Some(f) = sending.recv().await {
                    self.tx.send(f).unwrap();
                }
            });
            while let Some(f) = rx.recv().await {
                incoming.send(f).unwrap();
            }
            Ok(())
        })
    }
}
