use crate::errors::RSocketError;
use crate::frame::{self, Frame};
use crate::payload::{Payload, SetupPayload, SetupPayloadBuilder};
use crate::spi::{Flux, Mono, RSocket};
use crate::transport::{self, Acceptor, ClientTransport, DuplexSocket, Rx, Tx};
use futures::{Future, Stream};
use std::error::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;

#[derive(Clone)]
pub struct Client {
    socket: DuplexSocket,
}

pub struct ClientBuilder<T>
where
    T: Send + Sync + ClientTransport + 'static,
{
    transport: Option<T>,
    setup: SetupPayloadBuilder,
    responder: Option<fn() -> Box<dyn RSocket>>,
}

impl Client {
    fn new(socket: DuplexSocket) -> Client {
        Client { socket }
    }
    pub fn close(self) {
        self.socket.close();
    }
}

impl<T> ClientBuilder<T>
where
    T: Send + Sync + ClientTransport + 'static,
{
    pub(crate) fn new() -> ClientBuilder<T> {
        ClientBuilder {
            transport: None,
            responder: None,
            setup: SetupPayload::builder(),
        }
    }

    pub fn transport(mut self, transport: T) -> Self {
        self.transport = Some(transport);
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

    pub async fn start(mut self) -> Result<Client, Box<dyn Error + Send + Sync>> {
        match self.transport.take() {
            Some(tp) => {
                let (rcv_tx, rcv_rx) = mpsc::unbounded_channel::<Frame>();
                let (snd_tx, snd_rx) = mpsc::unbounded_channel::<Frame>();
                tokio::spawn(async move {
                    tp.attach(rcv_tx, snd_rx).await.unwrap();
                });
                let duplex_socket = DuplexSocket::new(1, snd_tx.clone()).await;
                let duplex_socket_clone = duplex_socket.clone();
                let acceptor = match self.responder {
                    Some(r) => Acceptor::Simple(Arc::new(r)),
                    None => Acceptor::Empty(),
                };
                tokio::spawn(async move {
                    duplex_socket_clone.event_loop(acceptor, rcv_rx).await;
                });
                let setup = self.setup.build();
                duplex_socket.setup(setup).await;
                Ok(Client::new(duplex_socket))
            }
            None => panic!("missing transport"),
        }
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
    fn request_stream(&self, req: Payload) -> Flux<Result<Payload, RSocketError>> {
        self.socket.request_stream(req)
    }
    fn request_channel(
        &self,
        reqs: Flux<Result<Payload, RSocketError>>,
    ) -> Flux<Result<Payload, RSocketError>> {
        self.socket.request_channel(reqs)
    }
}
