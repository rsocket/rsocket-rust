use crate::frame::Frame;
use std::future::Future;
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
