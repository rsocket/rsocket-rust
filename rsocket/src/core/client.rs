use std::marker::PhantomData;
use std::net::SocketAddr;
use std::pin::Pin;
use std::sync::Arc;
use std::time::Duration;

use async_trait::async_trait;
use futures::{future, FutureExt, Sink, SinkExt, Stream, StreamExt};
use tokio::sync::{mpsc, Mutex, Notify};

use crate::error::{RSocketError, ERR_CONN_CLOSED};
use crate::frame::{self, Frame};
use crate::payload::{Payload, SetupPayload, SetupPayloadBuilder};
use crate::runtime;
use crate::spi::{ClientResponder, Flux, RSocket};
use crate::transport::{
    self, Connection, DuplexSocket, FrameSink, FrameStream, Splitter, Transport,
};
use crate::Result;

#[derive(Clone)]
pub struct Client {
    closed: Arc<Notify>,
    socket: DuplexSocket,
    closing: mpsc::Sender<()>,
}

pub struct ClientBuilder<T, C> {
    transport: Option<T>,
    setup: SetupPayloadBuilder,
    responder: Option<ClientResponder>,
    closer: Option<Box<dyn FnMut() + Send + Sync>>,
    mtu: usize,
    _c: PhantomData<C>,
}

impl<T, C> ClientBuilder<T, C>
where
    T: Send + Sync + Transport<Conn = C>,
    C: Send + Sync + Connection,
{
    pub(crate) fn new() -> ClientBuilder<T, C> {
        ClientBuilder {
            transport: None,
            responder: None,
            setup: SetupPayload::builder(),
            closer: None,
            mtu: 0,
            _c: PhantomData,
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
}

impl<T, C> ClientBuilder<T, C>
where
    T: Send + Sync + Transport<Conn = C> + 'static,
    C: Send + Sync + Connection + 'static,
{
    pub async fn start(mut self) -> Result<Client> {
        let tp: T = self.transport.take().expect("missint transport");

        let splitter = if self.mtu == 0 {
            None
        } else {
            Some(Splitter::new(self.mtu))
        };

        let (snd_tx, mut snd_rx) = mpsc::unbounded_channel::<Frame>();
        let cloned_snd_tx = snd_tx.clone();
        let mut socket = DuplexSocket::new(1, snd_tx, splitter).await;

        let mut cloned_socket = socket.clone();

        if let Some(f) = self.responder {
            let responder = f();
            socket.bind_responder(responder).await;
        }

        let conn = tp.connect().await?;
        let (mut sink, mut stream) = conn.split();

        let setup = self.setup.build();

        // begin write loop
        let tick_period = setup.keepalive_interval();
        runtime::spawn(async move {
            loop {
                // send keepalive if timeout
                match tokio::time::timeout(tick_period, snd_rx.recv()).await {
                    Ok(Some(frame)) => {
                        if let frame::Body::Error(e) = frame.get_body_ref() {
                            if e.get_code() == ERR_CONN_CLOSED {
                                break;
                            }
                        }
                        if let Err(e) = sink.send(frame).await {
                            error!("write frame failed: {}", e);
                            break;
                        }
                    }
                    Ok(None) => break,
                    Err(_) => {
                        // keepalive
                        let keepalive_frame =
                            frame::Keepalive::builder(0, Frame::FLAG_RESPOND).build();
                        if let Err(e) = sink.send(keepalive_frame).await {
                            error!("write frame failed: {}", e);
                            break;
                        }
                    }
                }
            }
            println!("loop1: closed");
        });

        // begin read loop
        let closer = self.closer.take();
        let close_notify = Arc::new(Notify::new());
        let close_notify_clone = close_notify.clone();
        let (closing, mut closing_rx) = mpsc::channel::<()>(1);

        let (read_tx, mut read_rx) = mpsc::unbounded_channel::<Frame>();

        // read frames from stream, then writes into channel
        runtime::spawn(async move {
            loop {
                tokio::select! {
                    res = stream.next() => {
                        match res {
                            Some(next) => match next {
                                Ok(frame) => {
                                    if let Err(e) = read_tx.send(frame) {
                                        error!("forward frame failed: {}", e);
                                        break;
                                    }
                                }
                                Err(e) => {
                                    error!("read frame failed: {}", e);
                                    break;
                                }
                            }
                            None => break,
                        }
                    }
                    _ = closing_rx.recv() => {
                        break
                    }
                }
            }
        });

        // process frames
        runtime::spawn(async move {
            while let Some(next) = read_rx.recv().await {
                if let Err(e) = cloned_socket.dispatch(next, None).await {
                    error!("dispatch frame failed: {}", e);
                    break;
                }
            }

            // workaround: send a notify frame that the connection has been closed.
            let close_frame = frame::Error::builder(0, 0)
                .set_code(ERR_CONN_CLOSED)
                .build();
            if let Err(e) = cloned_snd_tx.send(close_frame) {
                debug!("send close notify frame failed: {}", e);
            }

            // notify client closed
            close_notify_clone.notify_one();

            // invoke on_close handler
            if let Some(mut invoke) = closer {
                invoke();
            }
        });

        socket.setup(setup).await;

        Ok(Client::new(socket, close_notify, closing))
    }
}

impl Client {
    fn new(socket: DuplexSocket, closed: Arc<Notify>, closing: mpsc::Sender<()>) -> Client {
        Client {
            socket,
            closed,
            closing,
        }
    }

    pub async fn wait_for_close(self) {
        self.closed.notified().await
    }
}

#[async_trait]
impl RSocket for Client {
    async fn metadata_push(&self, req: Payload) -> Result<()> {
        self.socket.metadata_push(req).await
    }

    async fn fire_and_forget(&self, req: Payload) -> Result<()> {
        self.socket.fire_and_forget(req).await
    }

    async fn request_response(&self, req: Payload) -> Result<Option<Payload>> {
        self.socket.request_response(req).await
    }

    fn request_stream(&self, req: Payload) -> Flux<Result<Payload>> {
        self.socket.request_stream(req)
    }

    fn request_channel(&self, reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        self.socket.request_channel(reqs)
    }
}
