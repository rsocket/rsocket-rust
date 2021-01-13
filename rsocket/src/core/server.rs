use std::error::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;

use futures::{SinkExt, StreamExt};
use tokio::sync::mpsc;

use crate::error::RSocketError;
use crate::frame::{self, Frame};
use crate::payload::SetupPayload;
use crate::runtime;
use crate::spi::{RSocket, ServerResponder};
use crate::transport::{Connection, DuplexSocket, ServerTransport, Splitter, Transport, MIN_MTU};
use crate::utils::EmptyRSocket;
use crate::Result;

pub struct ServerBuilder<T, C> {
    transport: Option<T>,
    on_setup: Option<ServerResponder>,
    start_handler: Option<Box<dyn FnMut() + Send + Sync>>,
    mtu: usize,
    _c: PhantomData<C>,
}

impl<T, C> ServerBuilder<T, C>
where
    T: Send + Sync + ServerTransport<Item = C>,
    C: Send + Sync + Transport,
{
    pub(crate) fn new() -> ServerBuilder<T, C> {
        ServerBuilder {
            transport: None,
            on_setup: None,
            start_handler: None,
            mtu: 0,
            _c: PhantomData,
        }
    }
    pub fn fragment(mut self, mtu: usize) -> Self {
        if mtu > 0 && mtu < MIN_MTU {
            panic!("invalid fragment mtu: at least {}!", MIN_MTU)
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
}

impl<T, C> ServerBuilder<T, C>
where
    T: Send + Sync + ServerTransport<Item = C> + 'static,
    C: Send + Sync + Transport + 'static,
{
    pub async fn serve(mut self) -> Result<()> {
        let mut server_transport = self.transport.take().expect("missing transport");
        // let acceptor = self.on_setup.map(|v| Acceptor::Generate(Arc::new(v)));

        let mtu = self.mtu;

        server_transport.start().await?;

        if let Some(mut invoke) = self.start_handler {
            invoke();
        }

        let acceptor = Arc::new(self.on_setup);
        while let Some(next) = server_transport.next().await {
            match next {
                Ok(tp) => {
                    let acceptor = acceptor.clone();
                    runtime::spawn(async move {
                        if let Err(e) = Self::on_transport(mtu, tp, acceptor).await {
                            error!("handle transport failed: {}", e);
                        }
                    });
                }
                Err(e) => {
                    error!("accept next transport failed: {}", e);
                }
            }
        }
        Ok(())
    }

    #[inline]
    async fn on_transport(mtu: usize, tp: C, acceptor: Arc<Option<ServerResponder>>) -> Result<()> {
        // Establish connection.
        let conn = tp.connect().await?;
        let (mut writer, mut reader) = conn.split();

        // Create frame splitter.
        let splitter = if mtu != 0 {
            Some(Splitter::new(mtu))
        } else {
            None
        };

        // Init duplex socket.
        let (snd_tx, mut snd_rx) = mpsc::unbounded_channel::<Frame>();
        let mut socket = DuplexSocket::new(0, snd_tx, splitter).await;

        // Begin loop for writing frames.
        runtime::spawn(async move {
            while let Some(frame) = snd_rx.recv().await {
                if let Err(e) = writer.send(frame).await {
                    error!("write frame failed: {}", e);
                    break;
                }
            }
        });

        let (read_tx, mut read_rx) = mpsc::unbounded_channel::<Frame>();

        runtime::spawn(async move {
            loop {
                match reader.next().await {
                    Some(Ok(frame)) => {
                        if let Err(e) = read_tx.send(frame) {
                            error!("forward frame failed: {}", e);
                            break;
                        }
                    }
                    Some(Err(e)) => {
                        error!("read frame failed: {}", e);
                        break;
                    }
                    None => {
                        break;
                    }
                }
            }
        });

        while let Some(frame) = read_rx.recv().await {
            if let Err(e) = socket.dispatch(frame, acceptor.as_ref().as_ref()).await {
                error!("dispatch frame failed: {}", e);
                break;
            }
        }

        Ok(())
    }
}
