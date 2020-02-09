use crate::errors::RSocketError;
use crate::frame::{self, Frame};
use crate::payload::SetupPayload;
use crate::runtime::{DefaultSpawner, Spawner};
use crate::spi::{EmptyRSocket, RSocket};
use crate::transport::{
    Acceptor, ClientTransport, DuplexSocket, FnAcceptorWithSetup, ServerTransport,
};
use futures::channel::mpsc;
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;

type FnStart = fn();

pub struct ServerBuilder<T, C>
where
    T: Send + Sync + ServerTransport<Item = C>,
    C: Send + Sync + ClientTransport,
{
    transport: Option<T>,
    on_setup: FnAcceptorWithSetup,
    start_handler: Option<FnStart>,
}

impl<T, C> ServerBuilder<T, C>
where
    T: Send + Sync + ServerTransport<Item = C> + 'static,
    C: Send + Sync + ClientTransport + 'static,
{
    pub(crate) fn new() -> ServerBuilder<T, C> {
        ServerBuilder {
            transport: None,
            on_setup: on_setup_noop,
            start_handler: None,
        }
    }

    pub fn acceptor(mut self, handler: FnAcceptorWithSetup) -> Self {
        self.on_setup = handler;
        self
    }

    pub fn on_start(mut self, hanlder: FnStart) -> Self {
        self.start_handler = Some(hanlder);
        self
    }

    pub fn transport(mut self, transport: T) -> Self {
        self.transport = Some(transport);
        self
    }

    pub async fn serve(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        self.serve_with_runtime(DefaultSpawner).await
    }

    pub async fn serve_with_runtime<R>(mut self, rt: R) -> Result<(), Box<dyn Error + Send + Sync>>
    where
        R: Send + Sync + Clone + Spawner + 'static,
    {
        let tp = self.transport.take().expect("missing transport");
        tp.start(self.start_handler, move |tp| {
            let cloned_rt = rt.clone();
            let setuper = Arc::new(self.on_setup);
            let (rcv_tx, rcv_rx) = mpsc::unbounded::<Frame>();
            let (snd_tx, snd_rx) = mpsc::unbounded::<Frame>();
            rt.spawn(async move {
                tp.attach(rcv_tx, snd_rx).await.unwrap();
            });
            rt.spawn(async move {
                let ds = DuplexSocket::new(cloned_rt, 0, snd_tx).await;
                let acceptor = Acceptor::Generate(setuper.clone());
                ds.event_loop(acceptor, rcv_rx).await;
            });
        })
        .await
    }
}

#[inline]
fn on_setup_noop(
    _setup: SetupPayload,
    _socket: Box<dyn RSocket>,
) -> Result<Box<dyn RSocket>, Box<dyn Error>> {
    Ok(Box::new(EmptyRSocket))
}
