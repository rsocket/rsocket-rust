use crate::error::RSocketError;
use crate::frame::Frame;
use crate::payload::SetupPayload;
use crate::spi::RSocket;
use futures::channel::{mpsc, oneshot};
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;

pub type Tx<T> = mpsc::UnboundedSender<T>;
pub type Rx<T> = mpsc::UnboundedReceiver<T>;

pub type TxOnce<T> = oneshot::Sender<T>;
pub type RxOnce<T> = oneshot::Receiver<T>;

pub(crate) fn new_tx_rx_once<T>() -> (TxOnce<T>, RxOnce<T>) {
    oneshot::channel()
}

pub(crate) fn new_tx_rx<T>() -> (Tx<T>, Rx<T>) {
    mpsc::unbounded()
}

pub trait ClientTransport {
    fn attach(
        self,
        incoming: Tx<Frame>,
        sending: Rx<Frame>,
        connected: Option<TxOnce<Result<(), RSocketError>>>,
    );
}

pub trait ServerTransport {
    type Item;

    fn start(
        self,
        starter: Option<Box<dyn FnMut() + Send + Sync>>,
        acceptor: impl Fn(Self::Item) + Send + Sync + 'static,
    ) -> Pin<Box<dyn Send + Future<Output = Result<(), Box<dyn Send + Sync + Error>>>>>
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
