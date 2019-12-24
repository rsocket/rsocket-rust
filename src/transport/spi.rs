use crate::frame::Frame;
use crate::payload::SetupPayload;
use crate::spi::RSocket;
use std::error::Error;
use std::future::Future;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

pub(crate) type Tx<T> = mpsc::UnboundedSender<T>;
pub(crate) type Rx<T> = mpsc::UnboundedReceiver<T>;
pub(crate) type TxOnce<T> = oneshot::Sender<T>;
pub(crate) type RxOnce<T> = oneshot::Receiver<T>;

pub(crate) fn new_tx_rx_once<T>() -> (TxOnce<T>, RxOnce<T>) {
    oneshot::channel()
}

pub(crate) fn new_tx_rx<T>() -> (Tx<T>, Rx<T>) {
    mpsc::unbounded_channel()
}

pub struct Transport {
    tx: Tx<Frame>,
    rx: Rx<Frame>,
}

impl Transport {
    pub fn new(tx: Tx<Frame>, rx: Rx<Frame>) -> Transport {
        Transport { tx, rx }
    }

    pub fn split(self) -> (Tx<Frame>, Rx<Frame>) {
        (self.tx, self.rx)
    }
}

pub type FnAcceptorWithSetup =
    fn(SetupPayload, Box<dyn RSocket>) -> Result<Box<dyn RSocket>, Box<dyn Error>>;

pub(crate) enum Acceptor {
    Simple(Arc<fn() -> Box<dyn RSocket>>),
    Generate(Arc<FnAcceptorWithSetup>),
    Empty(),
}
