use super::uri::URI;
use crate::errors::RSocketError;
use crate::frame::{self, Frame};
use crate::payload::SetupPayload;
use crate::spi::{EmptyRSocket, RSocket};
use crate::transport::ServerTransport;
use crate::transport::{
    Acceptor, ClientTransport, DuplexSocket, FnAcceptorWithSetup, TcpClientTransport,
};
use std::error::Error;
use std::future::Future;
use std::net::SocketAddr;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

type FnStart = fn();

pub struct ServerBuilder<'a> {
    uri: Option<&'a str>,
    on_setup: FnAcceptorWithSetup,
    start_handler: Option<FnStart>,
}

impl<'a> Default for ServerBuilder<'a> {
    fn default() -> Self {
        Self::new()
    }
}

impl<'a> ServerBuilder<'a> {
    pub fn new() -> ServerBuilder<'a> {
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

    pub fn transport(mut self, addr: &'a str) -> Self {
        self.uri = Some(addr);
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
        let server_tp = TcpServerTransport::new(addr);
        server_tp
            .start(on_start, move |tp| {
                let setuper = Arc::new(on_setup);
                let (rcv_tx, rcv_rx) = mpsc::unbounded_channel::<Frame>();
                let (snd_tx, snd_rx) = mpsc::unbounded_channel::<Frame>();
                tokio::spawn(async move {
                    tp.attach(rcv_tx, snd_rx).await.unwrap();
                });
                tokio::spawn(async move {
                    let ds = DuplexSocket::new(0, snd_tx).await;
                    let acceptor = Acceptor::Generate(setuper.clone());
                    ds.event_loop(acceptor, rcv_rx).await;
                });
            })
            .await

        // let mut listener = TcpListener::bind(&addr).await.unwrap();
        // debug!("listening on: {}", addr);
        // if let Some(it) = on_start {
        //     it();
        // }
        // loop {
        //     let (socket, _) = listener.accept().await.unwrap();
        //     let (rcv_tx, rcv_rx) = mpsc::unbounded_channel::<Frame>();
        //     let (snd_tx, snd_rx) = mpsc::unbounded_channel::<Frame>();
        //     let tp = TcpClientTransport::from(socket);
        //     tokio::spawn(async move {
        //         tp.attach(rcv_tx, snd_rx).await.unwrap();
        //     });
        //     let setuper = Arc::new(on_setup);
        //     let next_acceptor = move || Acceptor::Generate(setuper.clone());
        //     let ds = DuplexSocket::new(0, snd_tx.clone()).await;
        //     tokio::spawn(async move {
        //         let acceptor = next_acceptor();
        //         ds.event_loop(acceptor, rcv_rx).await;
        //     });
        // }
    }
}

struct TcpServerTransport {
    addr: SocketAddr,
}

impl TcpServerTransport {
    fn new(addr: SocketAddr) -> TcpServerTransport {
        TcpServerTransport { addr }
    }
}

impl ServerTransport for TcpServerTransport {
    type Item = TcpClientTransport;

    fn start(
        self,
        starter: Option<fn()>,
        acceptor: impl Fn(Self::Item) + Send + Sync + 'static,
    ) -> Pin<Box<dyn Send + Sync + Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>>
    where
        Self::Item: ClientTransport + Sized,
    {
        Box::pin(async move {
            let mut listener = TcpListener::bind(&self.addr).await.unwrap();
            debug!("listening on: {}", &self.addr);
            if let Some(bingo) = starter {
                bingo();
            }
            while let Ok((socket, _)) = listener.accept().await {
                let tp = TcpClientTransport::from(socket);
                acceptor(tp);
            }
            Ok(())
        })
    }
}

#[inline]
fn on_setup_noop(
    _setup: SetupPayload,
    _socket: Box<dyn RSocket>,
) -> Result<Box<dyn RSocket>, Box<dyn Error>> {
    Ok(Box::new(EmptyRSocket))
}
