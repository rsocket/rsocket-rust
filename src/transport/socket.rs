use super::misc::StreamID;
use super::spi::{Rx, Transport, Tx};
use crate::errors::{ErrorKind, RSocketError};
use crate::frame::{self, Body, Frame};
use crate::payload::{Payload, SetupPayload};
use crate::result::RSocketResult;
use crate::spi::{Acceptor, EmptyRSocket, RSocket, Single};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{SinkExt, StreamExt};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::sync::{mpsc, oneshot};

#[derive(Clone)]
pub(crate) struct DuplexSocket {
    seq: StreamID,
    responder: Responder,
    tx: Tx,
    handlers: Arc<Handlers>,
}

#[derive(Clone)]
struct Responder {
    inner: Arc<RwLock<Box<dyn RSocket>>>,
}

#[derive(Debug)]
enum Handler {
    Request(oneshot::Sender<Payload>),
    Stream(mpsc::Sender<Payload>),
}

#[derive(Debug)]
struct Handlers {
    map: RwLock<HashMap<u32, Handler>>,
}

impl DuplexSocket {
    pub fn new(first_stream_id: u32, tx: Tx) -> DuplexSocket {
        return DuplexSocket {
            seq: StreamID::from(first_stream_id),
            tx,
            responder: Responder::new(),
            handlers: Arc::new(Handlers::new()),
        };
    }

    pub async fn setup(&self, setup: SetupPayload) {
        let mut bu = frame::Setup::builder(0, 0);
        if let Some(s) = setup.data_mime_type() {
            bu = bu.set_mime_data(&s);
        }
        if let Some(s) = setup.metadata_mime_type() {
            bu = bu.set_mime_metadata(&s);
        }
        bu = bu.set_keepalive(setup.keepalive_interval());
        bu = bu.set_lifetime(setup.keepalive_lifetime());
        let (d, m) = setup.split();
        if let Some(b) = d {
            bu = bu.set_data(b);
        }
        if let Some(b) = m {
            bu = bu.set_metadata(b);
        }
        self.send_frame(bu.build()).await;
    }

    async fn send_frame(&self, sending: Frame) {
        let tx = self.tx.clone();
        tx.send(sending).unwrap();
    }

    pub async fn event_loop(&self, acceptor: Acceptor, mut rx: Rx) {
        loop {
            match rx.recv().await {
                Some(msg) => {
                    let sid = msg.get_stream_id();
                    let flag = msg.get_flag();
                    debug!("<--- RCV: {:?}", msg);
                    match msg.get_body() {
                        Body::Setup(v) => {
                            self.on_setup(&acceptor, sid, flag, SetupPayload::from(v))
                        }
                        Body::RequestFNF(v) => {
                            let input = Payload::from(v);
                            self.on_fnf(sid, flag, input).await;
                        }
                        Body::Payload(v) => {
                            let input = Payload::from(v);
                            self.on_payload(sid, flag, input).await;
                        }
                        Body::RequestResponse(v) => {
                            let input = Payload::from(v);
                            self.on_request_response(sid, flag, input).await;
                        }
                        Body::MetadataPush(v) => {
                            let input = Payload::from(v);
                            self.on_metadata_push(input).await;
                        }
                        Body::Keepalive(v) => {
                            if flag & frame::FLAG_RESPOND != 0 {
                                debug!("got keepalive: {:?}", v);
                                self.on_keepalive(v).await;
                            }
                        }
                        _ => (),
                    }
                }
                None => {
                    break;
                }
            }
        }
    }

    #[inline]
    async fn on_payload(&self, sid: u32, flag: u16, input: Payload) {
        // pick handler
        let handlers = self.handlers.clone();
        let mut senders = handlers.map.write().unwrap();
        let handler = senders.remove(&sid).unwrap();

        let mut tx1: Option<oneshot::Sender<Payload>> = None;
        let mut tx2: Option<mpsc::Sender<Payload>> = None;

        // fire event!
        match handler {
            Handler::Request(sender) => {
                tx1 = Some(sender);
            }
            Handler::Stream(sender) => {
                if flag & frame::FLAG_NEXT != 0 {
                    tx2 = Some(sender.clone());
                }
                if flag & frame::FLAG_COMPLETE == 0 {
                    senders.insert(sid, Handler::Stream(sender));
                }
            }
        };
        if let Some(sender) = tx1 {
            sender.send(input).unwrap();
        }
        // TODO: support mpsc
        // else if let Some(sender) = tx2 {
        //     sender.send(input).await.unwrap();
        // }
    }

    #[inline]
    fn on_setup(&self, acceptor: &Acceptor, sid: u32, flag: u16, setup: SetupPayload) {
        match acceptor {
            Acceptor::Generate(gen) => {
                self.responder.set(gen(setup, Box::new(self.clone())));
            }
            Acceptor::Empty() => {
                self.responder.set(Box::new(EmptyRSocket));
            }
            // TODO: How to direct???
            // Acceptor::Direct(sk) => self.responder.set(sk),
            _ => (),
        }
    }

    #[inline]
    async fn on_fnf(&self, sid: u32, flag: u16, input: Payload) {
        match self.responder.clone().fire_and_forget(input).await {
            Ok(()) => (),
            Err(e) => error!("respoond fnf failed: {}", e),
        }
    }

    #[inline]
    async fn on_request_response(&self, sid: u32, _flag: u16, input: Payload) {
        let responder = self.responder.clone();
        let tx = self.tx.clone();
        let sending = match responder.request_response(input).await {
            Ok(it) => {
                let (d, m) = it.split();
                let mut bu = frame::Payload::builder(sid, frame::FLAG_COMPLETE);
                if let Some(b) = d {
                    bu = bu.set_data(b);
                }
                if let Some(b) = m {
                    bu = bu.set_metadata(b);
                }
                bu.build()
            }
            Err(e) => frame::Error::builder(sid, 0)
                .set_code(frame::ERR_APPLICATION)
                .set_data(Bytes::from("TODO: should be error details"))
                .build(),
        };
        tx.send(sending).unwrap();
    }

    #[inline]
    async fn on_metadata_push(&self, input: Payload) {
        if let Err(e) = self.responder.clone().metadata_push(input).await {
            error!("metadata_push failed: {}", e);
        }
    }

    #[inline]
    async fn on_keepalive(&self, keepalive: frame::Keepalive) {
        let tx = self.tx.clone();
        let (data, _) = keepalive.split();
        let mut sending = frame::Keepalive::builder(0, 0);
        if let Some(b) = data {
            sending = sending.set_data(b);
        }
        tx.send(sending.build()).unwrap();
    }
}

impl RSocket for DuplexSocket {
    fn metadata_push(&self, req: Payload) -> Single<()> {
        unimplemented!();
    }
    fn fire_and_forget(&self, req: Payload) -> Single<()> {
        unimplemented!();
    }
    fn request_response(&self, req: Payload) -> Single<Payload> {
        unimplemented!();
    }
}

impl Handlers {
    fn new() -> Handlers {
        Handlers {
            map: RwLock::new(HashMap::new()),
        }
    }
}

impl From<Box<dyn RSocket>> for Responder {
    fn from(input: Box<dyn RSocket>) -> Responder {
        Responder {
            inner: Arc::new(RwLock::new(input)),
        }
    }
}

impl Responder {
    fn new() -> Responder {
        let bx = Box::new(EmptyRSocket);
        Responder {
            inner: Arc::new(RwLock::new(bx)),
        }
    }

    fn set(&self, rs: Box<dyn RSocket>) {
        let inner = self.inner.clone();
        let mut v = inner.write().unwrap();
        *v = rs;
    }
}

impl RSocket for Responder {
    fn metadata_push(&self, req: Payload) -> Single<()> {
        let r = self.inner.read().unwrap();
        (*r).metadata_push(req)
    }

    fn fire_and_forget(&self, req: Payload) -> Single<()> {
        let r = self.inner.read().unwrap();
        (*r).fire_and_forget(req)
    }

    fn request_response(&self, req: Payload) -> Single<Payload> {
        let r = self.inner.read().unwrap();
        (*r).request_response(req)
    }
}
