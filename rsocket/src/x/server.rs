use crate::error::RSocketError;
use crate::frame::{self, Frame};
use crate::payload::SetupPayload;
use crate::runtime::{DefaultSpawner, Spawner};
use crate::spi::{EmptyRSocket, RSocket, ServerResponder};
use crate::transport::{self, Acceptor, ClientTransport, DuplexSocket, ServerTransport, Splitter};
use futures::channel::{mpsc, oneshot};
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;

pub struct ServerBuilder<T, C>
where
    T: Send + Sync + ServerTransport<Item = C>,
    C: Send + Sync + ClientTransport,
{
    transport: Option<T>,
    on_setup: Option<ServerResponder>,
    start_handler: Option<Box<dyn FnMut() + Send + Sync>>,
    mtu: usize,
}

impl<T, C> ServerBuilder<T, C>
where
    T: Send + Sync + ServerTransport<Item = C> + 'static,
    C: Send + Sync + ClientTransport + 'static,
{
    pub(crate) fn new() -> ServerBuilder<T, C> {
        ServerBuilder {
            transport: None,
            on_setup: None,
            start_handler: None,
            mtu: 0,
        }
    }

    pub fn fragment(mut self, mtu: usize) -> Self {
        if mtu > 0 && mtu < transport::MIN_MTU {
            panic!("invalid fragment mtu: at least {}!", transport::MIN_MTU)
        }
        self.mtu = mtu;
        self
    }

    pub fn acceptor(mut self, handler: ServerResponder) -> Self {
        self.on_setup = Some(handler);
        self
    }

    pub fn on_start(mut self, hanlder: Box<dyn FnMut() + Send + Sync>) -> Self {
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
        let starter = self.start_handler;
        let acceptor = match self.on_setup {
            Some(v) => Some(Acceptor::Generate(Arc::new(v))),
            None => None,
        };

        let mtu = self.mtu;

        tp.start(starter, move |tp| {
            let cloned_rt = rt.clone();
            let (rcv_tx, rcv_rx) = mpsc::unbounded::<Frame>();
            let (snd_tx, snd_rx) = mpsc::unbounded::<Frame>();
            tp.attach(rcv_tx, snd_rx, None);
            let acceptor = acceptor.clone();

            rt.spawn(async move {
                let splitter = if mtu == 0 {
                    None
                } else {
                    Some(Splitter::new(mtu))
                };
                let ds = DuplexSocket::new(cloned_rt, 0, snd_tx, splitter).await;
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
