use super::uri::URI;
use crate::errors::RSocketError;
use crate::frame::{self, Frame};
use crate::payload::{Payload, SetupPayload, SetupPayloadBuilder};
use crate::spi::{Flux, Mono, RSocket};
use crate::transport::{self, Acceptor, DuplexSocket};
use futures::{Future, Stream};
use std::error::Error;
use std::net::SocketAddr;
use std::result::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct Client {
    socket: DuplexSocket,
}

pub struct ClientBuilder {
    uri: Option<String>,
    setup: SetupPayloadBuilder,
    responder: Option<fn() -> Box<dyn RSocket>>,
}

impl Client {
    fn new(socket: DuplexSocket) -> Client {
        Client { socket }
    }

    pub fn builder() -> ClientBuilder {
        ClientBuilder::new()
    }

    pub fn close(self) {
        self.socket.close();
    }
}

impl ClientBuilder {
    fn new() -> ClientBuilder {
        ClientBuilder {
            uri: None,
            responder: None,
            setup: SetupPayload::builder(),
        }
    }

    pub fn transport(mut self, uri: &str) -> Self {
        self.uri = Some(uri.to_string());
        self
    }

    pub fn setup(mut self, setup: Payload) -> Self {
        let (d, m) = setup.split();
        if let Some(b) = d {
            self.setup = self.setup.set_data(b);
        }
        if let Some(b) = m {
            self.setup = self.setup.set_metadata(b);
        }
        self
    }

    pub fn keepalive(
        mut self,
        tick_period: Duration,
        ack_timeout: Duration,
        missed_acks: u64,
    ) -> Self {
        self.setup = self
            .setup
            .set_keepalive(tick_period, ack_timeout, missed_acks);
        self
    }

    pub fn mime_type(mut self, metadata_mime_type: &str, data_mime_type: &str) -> Self {
        self = self.metadata_mime_type(metadata_mime_type);
        self = self.data_mime_type(data_mime_type);
        self
    }

    pub fn data_mime_type(mut self, mime_type: &str) -> Self {
        self.setup = self.setup.set_data_mime_type(mime_type);
        self
    }

    pub fn metadata_mime_type(mut self, mime_type: &str) -> Self {
        self.setup = self.setup.set_metadata_mime_type(mime_type);
        self
    }

    pub fn acceptor(mut self, acceptor: fn() -> Box<dyn RSocket>) -> Self {
        self.responder = Some(acceptor);
        self
    }
    pub async fn start(self) -> Result<Client, Box<dyn Error>> {
        // TODO: process error
        let uri = self.uri.unwrap();
        match URI::parse(&uri) {
            Ok(u) => match u {
                URI::Tcp(vv) => Self::start_tcp(vv, self.responder, self.setup).await,
                _ => unimplemented!(),
            },
            Err(e) => Err(e),
        }
    }

    #[inline]
    async fn start_tcp(
        addr: SocketAddr,
        responder: Option<fn() -> Box<dyn RSocket>>,
        sb: SetupPayloadBuilder,
    ) -> Result<Client, Box<dyn Error>> {
        let socket = transport::tcp::connect(&addr);
        let (rcv_tx, rcv_rx) = mpsc::unbounded_channel::<Frame>();
        let (snd_tx, snd_rx) = mpsc::unbounded_channel::<Frame>();
        tokio::spawn(async move { crate::transport::tcp::process(socket, snd_rx, rcv_tx).await });
        let duplex_socket = DuplexSocket::new(1, snd_tx.clone());
        let duplex_socket_clone = duplex_socket.clone();

        tokio::spawn(async move {
            let acceptor = if let Some(r) = responder {
                Acceptor::Simple(Arc::new(r))
            } else {
                Acceptor::Empty()
            };
            duplex_socket_clone.event_loop(acceptor, rcv_rx).await;
        });
        let setup = sb.build();
        duplex_socket.setup(setup).await;
        Ok(Client::new(duplex_socket))
    }
}

impl RSocket for Client {
    fn metadata_push(&self, req: Payload) -> Mono<()> {
        self.socket.metadata_push(req)
    }
    fn fire_and_forget(&self, req: Payload) -> Mono<()> {
        self.socket.fire_and_forget(req)
    }
    fn request_response(&self, req: Payload) -> Mono<Result<Payload, RSocketError>> {
        self.socket.request_response(req)
    }
    fn request_stream(&self, req: Payload) -> Flux<Payload> {
        self.socket.request_stream(req)
    }
    fn request_channel(&self, reqs: Flux<Payload>) -> Flux<Payload> {
        self.socket.request_channel(reqs)
    }
}
