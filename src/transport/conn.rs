use super::misc::{Handler, Handlers, StreamID};
use super::spi::{Rx, Transport, Tx};
use crate::errors::{ErrorKind, RSocketError};
use crate::frame::{Body, Frame};
use crate::payload::{Payload, SetupPayload};
use crate::result::RSocketResult;
use crate::spi::{Acceptor, EmptyRSocket, RSocket, Single};
use futures::{SinkExt, StreamExt};
use std::env;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Mutex, RwLock};
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::sync::mpsc;

#[derive(Clone)]
struct Responder {
    inner: Arc<RwLock<Box<dyn RSocket>>>,
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

#[derive(Clone)]
pub struct DuplexSocket {
    seq: StreamID,
    responder: Responder,
    tx: Tx,
    handlers: Arc<Handlers>,
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
        // TODO: xxx
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
                            self.respond_fnf(sid, flag, input).await;
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
    fn on_setup(&self, acceptor: &Acceptor, sid: u32, flag: u16, setup: SetupPayload) {
        match acceptor {
            Acceptor::Generate(gen) => {
                // TODO: gen acceptor
                self.responder.set(gen(setup, Box::new(self.clone())));
            }
            _ => (),
        }
    }

    #[inline]
    async fn respond_fnf(&self, sid: u32, flag: u16, input: Payload) {
        match self.responder.clone().fire_and_forget(input).await {
            Ok(()) => (),
            Err(e) => error!("respoond fnf failed: {}", e),
        }
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
