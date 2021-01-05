use crate::error::RSocketError;
use crate::frame::{self, Frame};
use crate::payload::SetupPayload;
use crate::runtime;
use crate::spi::{RSocket, ServerResponder};
use crate::transport::{
    Acceptor, Connection, DuplexSocket, ServerTransport, Splitter, Transport, MIN_MTU,
};
use crate::utils::EmptyRSocket;
use crate::Result;
use std::error::Error;
use std::future::Future;
use std::marker::PhantomData;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;

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
        let acceptor = self.on_setup.map(|v| Acceptor::Generate(Arc::new(v)));

        let mtu = self.mtu;

        server_transport.start().await?;

        if let Some(mut invoke) = self.start_handler {
            invoke();
        }

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
    async fn on_transport(mtu: usize, tp: C, acceptor: Option<Acceptor>) -> Result<()> {
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
        let (snd_tx, mut snd_rx) = mpsc::channel::<Frame>(super::CHANNEL_SIZE);
        let mut socket = DuplexSocket::new(0, snd_tx, splitter).await;

        // Begin loop for writing frames.
        runtime::spawn(async move {
            while let Some(frame) = snd_rx.recv().await {
                if let Err(e) = writer.write(frame).await {
                    error!("write frame failed: {}", e);
                    break;
                }
            }
        });

        loop {
            match reader.read().await {
                Some(Ok(frame)) => {
                    if let Err(e) = socket.dispatch(frame, &acceptor).await {
                        error!("dispatch incoming frame failed: {}", e);
                        break;
                    }
                }
                Some(Err(e)) => {
                    error!("read next frame failed: {}", e);
                    break;
                }
                None => {
                    break;
                }
            }
        }

        Ok(())
    }
}
