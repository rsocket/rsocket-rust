use crate::frame::Frame;
use crate::payload::SetupPayload;
use crate::spi::RSocket;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::{mpsc, oneshot};

pub type Tx<T> = mpsc::UnboundedSender<T>;
pub type Rx<T> = mpsc::UnboundedReceiver<T>;
pub(crate) type TxOnce<T> = oneshot::Sender<T>;
pub(crate) type RxOnce<T> = oneshot::Receiver<T>;

pub(crate) fn new_tx_rx_once<T>() -> (TxOnce<T>, RxOnce<T>) {
    oneshot::channel()
}

pub(crate) fn new_tx_rx<T>() -> (Tx<T>, Rx<T>) {
    mpsc::unbounded_channel()
}

pub trait ClientTransport {
    fn attach(
        self,
        incoming: Tx<Frame>,
        sending: Rx<Frame>,
    ) -> Pin<Box<dyn Sync + Send + Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>>;
}

pub trait ServerTransport {
    type Item;
    fn start(
        self,
        starter: Option<fn()>,
        acceptor: impl Fn(Self::Item) + Send + Sync + 'static,
    ) -> Pin<Box<dyn Sync + Send + Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>>
    where
        Self::Item: ClientTransport + Sized;
}

pub type FnAcceptorWithSetup =
    fn(SetupPayload, Box<dyn RSocket>) -> Result<Box<dyn RSocket>, Box<dyn Error>>;

pub(crate) enum Acceptor {
    Simple(Arc<fn() -> Box<dyn RSocket>>),
    Generate(Arc<FnAcceptorWithSetup>),
    Empty(),
}
