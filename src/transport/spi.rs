use crate::frame::Frame;
use crate::payload::SetupPayload;
use crate::spi::RSocket;
use std::future::Future;
use std::sync::Arc;
use tokio::sync::mpsc::{Receiver, Sender, UnboundedReceiver, UnboundedSender};

pub type Tx = UnboundedSender<Frame>;
pub type Rx = UnboundedReceiver<Frame>;

pub struct Transport {
    tx: UnboundedSender<Frame>,
    rx: UnboundedReceiver<Frame>,
}

impl Transport {
    pub fn new(tx: Tx, rx: Rx) -> Transport {
        Transport { tx, rx }
    }

    pub fn split(self) -> (Tx, Rx) {
        (self.tx, self.rx)
    }
}

type FnAcceptorWithSetup = fn(SetupPayload, Box<dyn RSocket>) -> Box<dyn RSocket>;

pub(crate) enum Acceptor {
    Simple(Arc<fn() -> Box<dyn RSocket>>),
    Generate(Arc<FnAcceptorWithSetup>),
    Empty(),
}
