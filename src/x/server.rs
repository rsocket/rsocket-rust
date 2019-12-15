use super::uri::URI;
use crate::errors::RSocketError;
use crate::frame::{self, Frame};
use crate::payload::SetupPayload;
use crate::spi::{EmptyRSocket, RSocket};
use crate::transport::{Acceptor, DuplexSocket, FnAcceptorWithSetup};
use std::error::Error;
use std::net::SocketAddr;
use std::result::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

type FnStart = fn();

pub struct ServerBuilder {
    uri: Option<String>,
    on_setup: FnAcceptorWithSetup,
    start_handler: Option<FnStart>,
}

impl Default for ServerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerBuilder {
    pub fn new() -> ServerBuilder {
        ServerBuilder {
            uri: None,
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

    pub fn transport(mut self, addr: &str) -> Self {
        self.uri = Some(String::from(addr));
        self
    }

    pub async fn serve(self) -> Result<(), Box<dyn Error + Send + Sync>> {
        // TODO: process error
        let s = self.uri.unwrap();
        match URI::parse(&s) {
            Ok(u) => match u {
                URI::Tcp(addr) => Self::serve_tcp(addr, self.on_setup, self.start_handler).await,
                _ => unimplemented!(),
            },
            Err(e) => Err(e),
        }
    }

    #[inline]
    async fn serve_tcp(
        addr: SocketAddr,
        on_setup: FnAcceptorWithSetup,
        on_start: Option<FnStart>,
    ) -> Result<(), Box<dyn Error + Send + Sync>> {
        let mut listener = TcpListener::bind(&addr).await.unwrap();
        debug!("listening on: {}", addr);
        if let Some(it) = on_start {
            it();
        }
        loop {
            let (socket, _) = listener.accept().await.unwrap();
            let (rcv_tx, rcv_rx) = mpsc::unbounded_channel::<Frame>();
            let (snd_tx, snd_rx) = mpsc::unbounded_channel::<Frame>();
            tokio::spawn(
                async move { crate::transport::tcp::process(socket, snd_rx, rcv_tx).await },
            );
            let setuper = Arc::new(on_setup);
            let next_acceptor = move || Acceptor::Generate(setuper.clone());
            let ds = DuplexSocket::new(0, snd_tx.clone()).await;
            tokio::spawn(async move {
                let acceptor = next_acceptor();
                ds.event_loop(acceptor, rcv_rx).await;
            });
        }
    }
}

#[inline]
fn on_setup_noop(
    _setup: SetupPayload,
    _socket: Box<dyn RSocket>,
) -> Result<Box<dyn RSocket>, Box<dyn Error>> {
    Ok(Box::new(EmptyRSocket))
}
