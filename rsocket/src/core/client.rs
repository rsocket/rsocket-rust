use crate::error::RSocketError;
use crate::frame::{self, Frame};
use crate::payload::{Payload, SetupPayload, SetupPayloadBuilder};
use crate::runtime;
use crate::spi::{ClientResponder, Flux, Mono, RSocket};
use crate::transport::{
    self, Acceptor, Connection, DuplexSocket, Reader, Splitter, Transport, Writer,
};
use crate::Result;
use futures::{future, FutureExt, SinkExt, StreamExt};
use std::error::Error;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::{mpsc, Mutex, Notify};

#[derive(Clone)]
pub struct Client {
    socket: DuplexSocket,
}

pub struct ClientBuilder<T, C>
where
    T: Send + Sync + Transport<Conn = C> + 'static,
    C: Send + Sync + Connection + 'static,
{
    transport: Option<T>,
    setup: SetupPayloadBuilder,
    responder: Option<ClientResponder>,
    closer: Option<Box<dyn FnMut() + Send + Sync>>,
    mtu: usize,
}

impl<T, C> ClientBuilder<T, C>
where
    T: Send + Sync + Transport<Conn = C> + 'static,
    C: Send + Sync + Connection + 'static,
{
    pub(crate) fn new() -> ClientBuilder<T, C> {
        ClientBuilder {
            transport: None,
            responder: None,
            setup: SetupPayload::builder(),
            closer: None,
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

    pub fn transport(mut self, transport: T) -> Self {
        self.transport = Some(transport);
        self
    }

    pub fn setup(mut self, setup: Payload) -> Self {
        let (d, m) = setup.split();
        self.setup = self.setup.set_data_bytes(d);
        self.setup = self.setup.set_metadata_bytes(m);
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

    pub fn acceptor(mut self, acceptor: ClientResponder) -> Self {
        self.responder = Some(acceptor);
        self
    }

    pub fn on_close(mut self, callback: Box<dyn FnMut() + Sync + Send>) -> Self {
        self.closer = Some(callback);
        self
    }

    pub async fn start(mut self) -> Result<Client> {
        let tp: T = self.transport.take().expect("missint transport");

        let splitter = if self.mtu == 0 {
            None
        } else {
            Some(Splitter::new(self.mtu))
        };

        let (snd_tx, mut snd_rx) = mpsc::channel::<Frame>(super::CHANNEL_SIZE);
        let mut socket = DuplexSocket::new(1, snd_tx, splitter).await;

        let mut cloned_socket = socket.clone();
        let acceptor: Option<Acceptor> = match self.responder {
            Some(it) => Some(Acceptor::Simple(Arc::new(it))),
            None => None,
        };

        let conn = tp.connect().await?;
        let (mut sink, mut stream) = conn.split();

        runtime::spawn(async move {
            while let Some(frame) = snd_rx.next().await {
                if let Err(e) = (&mut sink).write(frame).await {
                    error!("write frame failed: {}", e);
                    break;
                }
            }
        });

        let closer = self.closer.take();
        runtime::spawn(async move {
            while let Some(next) = stream.read().await {
                match next {
                    Ok(frame) => {
                        if let Err(e) = cloned_socket.dispatch(frame, &acceptor).await {
                            error!("dispatch frame failed: {}", e);
                            break;
                        }
                    }
                    Err(e) => {
                        error!("read next frame failed: {}", e);
                        break;
                    }
                }
            }
            if let Some(mut invoke) = closer {
                invoke();
            }
        });

        socket.setup(self.setup.build()).await;
        Ok(Client::from(socket))
    }
}

impl From<DuplexSocket> for Client {
    fn from(socket: DuplexSocket) -> Client {
        Client { socket }
    }
}

impl Client {
    pub fn close(self) {
        // TODO: support close
    }
}

impl RSocket for Client {
    fn metadata_push(&self, req: Payload) -> Mono<()> {
        self.socket.metadata_push(req)
    }

    fn fire_and_forget(&self, req: Payload) -> Mono<()> {
        self.socket.fire_and_forget(req)
    }

    fn request_response(&self, req: Payload) -> Mono<Result<Payload>> {
        self.socket.request_response(req)
    }

    fn request_stream(&self, req: Payload) -> Flux<Result<Payload>> {
        self.socket.request_stream(req)
    }

    fn request_channel(&self, reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        self.socket.request_channel(reqs)
    }
}
