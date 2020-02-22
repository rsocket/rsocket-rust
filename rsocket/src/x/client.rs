use crate::error::RSocketError;
use crate::frame::{self, Frame};
use crate::payload::{Payload, SetupPayload, SetupPayloadBuilder};
use crate::runtime::{DefaultSpawner, Spawner};
use crate::spi::{Flux, Mono, RSocket};
use crate::transport::{self, Acceptor, ClientTransport, DuplexSocket, Rx, Tx};
use futures::channel::{mpsc, oneshot};
use futures::{Future, Stream};
use std::error::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;
use std::time::Duration;

#[derive(Clone)]
pub struct Client<R>
where
    R: Send + Sync + Clone + Spawner + 'static,
{
    socket: DuplexSocket<R>,
}

pub struct ClientBuilder<T>
where
    T: Send + Sync + ClientTransport + 'static,
{
    transport: Option<T>,
    setup: SetupPayloadBuilder,
    responder: Option<fn() -> Box<dyn RSocket>>,
}

impl<R> Client<R>
where
    R: Send + Sync + Clone + Spawner + 'static,
{
    fn new(socket: DuplexSocket<R>) -> Client<R> {
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

    pub async fn start(self) -> Result<Client<DefaultSpawner>, Box<dyn Error + Send + Sync>> {
        self.start_with_runtime(DefaultSpawner).await
    }

    pub async fn start_with_runtime<R>(
        mut self,
        rt: R,
    ) -> Result<Client<R>, Box<dyn Error + Send + Sync>>
    where
        R: Send + Sync + Clone + Spawner + 'static,
    {
        let tp = self.transport.take().expect("missint transport");
        let cloned_rt = rt.clone();
        let (rcv_tx, rcv_rx) = mpsc::unbounded::<Frame>();
        let (snd_tx, snd_rx) = mpsc::unbounded::<Frame>();
        let (connected_tx, connected_rx) = oneshot::channel::<Result<(), RSocketError>>();
        tp.attach(rcv_tx, snd_rx, Some(connected_tx));
        connected_rx.await??;

        let duplex_socket = DuplexSocket::new(rt, 1, snd_tx.clone()).await;
        let cloned_duplex_socket = duplex_socket.clone();
        let acceptor = match self.responder {
            Some(r) => Acceptor::Simple(Arc::new(r)),
            None => Acceptor::Empty(),
        };
        cloned_rt.spawn(async move {
            cloned_duplex_socket.event_loop(acceptor, rcv_rx).await;
        });
        let setup = self.setup.build();
        duplex_socket.setup(setup).await;
        Ok(Client::new(duplex_socket))
    }
}

impl<R> RSocket for Client<R>
where
    R: Send + Sync + Clone + Spawner + 'static,
{
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
