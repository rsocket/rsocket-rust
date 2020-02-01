use super::misc::{self, Counter, StreamID};
use super::spi::*;
use crate::errors::{self, ErrorKind, RSocketError};
use crate::frame::{self, Body, Frame};
use crate::misc::RSocketResult;
use crate::payload::{Payload, SetupPayload};
use crate::spi::{EmptyRSocket, Flux, Mono, RSocket};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{future, Sink, SinkExt, Stream, StreamExt};
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::ptr;
use std::result::Result;
use std::sync::{Arc, RwLock};
use tokio::prelude::*;
use tokio::sync::Mutex;

#[derive(Clone)]
pub(crate) struct DuplexSocket {
    seq: StreamID,
    responder: Responder,
    tx: Tx<Frame>,
    handlers: Arc<Mutex<HashMap<u32, Handler>>>,
    canceller: Tx<u32>,
}

#[derive(Clone)]
struct Responder {
    inner: Arc<RwLock<Box<dyn RSocket>>>,
}

#[derive(Debug)]
enum Handler {
    ReqRR(TxOnce<Result<Payload, RSocketError>>),
    ResRR(Counter),
    ReqRS(Tx<Result<Payload, RSocketError>>),
    ReqRC(Tx<Result<Payload, RSocketError>>),
}

impl DuplexSocket {
    pub(crate) async fn new(first_stream_id: u32, tx: Tx<Frame>) -> DuplexSocket {
        let (canceller_tx, canceller_rx) = new_tx_rx::<u32>();
        let ds = DuplexSocket {
            seq: StreamID::from(first_stream_id),
            tx,
            canceller: canceller_tx,
            responder: Responder::new(),
            handlers: Arc::new(Mutex::new(HashMap::new())),
        };

        let ds2 = ds.clone();
        tokio::spawn(async move {
            ds2.loop_canceller(canceller_rx).await;
        });
        ds
    }

    pub(crate) fn close(self) {
        drop(self.tx);
    }

    pub(crate) async fn setup(&self, setup: SetupPayload) {
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
        self.tx.send(bu.build()).unwrap();
    }

    #[inline]
    async fn register_handler(&self, sid: u32, handler: Handler) {
        let mut handlers = self.handlers.lock().await;
        (*handlers).insert(sid, handler);
    }

    #[inline]
    pub(crate) async fn loop_canceller(&self, mut rx: Rx<u32>) {
        while let Some(sid) = rx.recv().await {
            let mut handlers = self.handlers.lock().await;
            (*handlers).remove(&sid);
        }
    }

    pub(crate) async fn event_loop(&self, acceptor: Acceptor, mut rx: Rx<Frame>) {
        while let Some(msg) = rx.recv().await {
            let sid = msg.get_stream_id();
            let flag = msg.get_flag();
            misc::debug_frame(false, &msg);
            match msg.get_body() {
                Body::Setup(v) => {
                    if let Err(e) = self.on_setup(&acceptor, sid, flag, SetupPayload::from(v)) {
                        let errmsg = format!("{}", e);
                        let sending = frame::Error::builder(0, 0)
                            .set_code(errors::ERR_REJECT_SETUP)
                            .set_data(Bytes::from(errmsg))
                            .build();
                        self.tx.send(sending).unwrap();
                        return;
                    }
                }
                Body::Resume(v) => {
                    // TODO: support resume
                }
                Body::ResumeOK(v) => {
                    // TODO: support resume ok
                }
                Body::MetadataPush(v) => {
                    let input = Payload::from(v);
                    self.on_metadata_push(input).await;
                }
                Body::RequestFNF(v) => {
                    let input = Payload::from(v);
                    self.on_fire_and_forget(sid, flag, input).await;
                }
                Body::RequestResponse(v) => {
                    let input = Payload::from(v);
                    self.on_request_response(sid, flag, input).await;
                }
                Body::RequestStream(v) => {
                    let input = Payload::from(v);
                    self.on_request_stream(sid, flag, input).await;
                }
                Body::RequestChannel(v) => {
                    let input = Payload::from(v);
                    self.on_request_channel(sid, flag, input).await;
                }
                Body::Payload(v) => {
                    let input = Payload::from(v);
                    self.on_payload(sid, flag, input).await;
                }
                Body::Keepalive(v) => {
                    if flag & frame::FLAG_RESPOND != 0 {
                        debug!("got keepalive: {:?}", v);
                        self.on_keepalive(v).await;
                    }
                }
                Body::RequestN(v) => {
                    // TODO: support RequestN
                }
                Body::Error(v) => {
                    // TODO: support error
                    self.on_error(sid, flag, v).await;
                }
                Body::Cancel() => {
                    self.on_cancel(sid, flag).await;
                }
                Body::Lease(v) => {
                    // TODO: support Lease
                }
            }
        }
    }

    #[inline]
    async fn on_error(&self, sid: u32, flag: u16, input: frame::Error) {
        // pick handler
        let mut handlers = self.handlers.lock().await;
        if let Some(handler) = (*handlers).remove(&sid) {
            let kind = ErrorKind::Internal(input.get_code(), input.get_data_utf8());
            let e = Err(RSocketError::from(kind));
            match handler {
                Handler::ReqRR(tx) => tx.send(e).unwrap(),
                Handler::ResRR(_) => unreachable!(),
                Handler::ReqRS(tx) => tx.send(e).unwrap(),
                _ => unimplemented!(),
            }
        }
    }

    #[inline]
    async fn on_cancel(&self, sid: u32, _flag: u16) {
        let mut handlers = self.handlers.lock().await;
        if let Some(handler) = (*handlers).remove(&sid) {
            let e = Err(RSocketError::from(ErrorKind::Cancelled()));
            match handler {
                Handler::ReqRR(sender) => {
                    info!("REQUEST_RESPONSE {} cancelled!", sid);
                    sender.send(e).unwrap();
                }
                Handler::ResRR(c) => {
                    let lefts = c.count_down();
                    info!("REQUEST_RESPONSE {} cancelled: lefts={}", sid, lefts);
                }
                Handler::ReqRS(sender) => {
                    info!("REQUEST_STREAM {} cancelled!", sid);
                }
                Handler::ReqRC(sender) => {
                    info!("REQUEST_CHANNEL {} cancelled!", sid);
                }
            };
        }
    }

    #[inline]
    async fn on_payload(&self, sid: u32, flag: u16, input: Payload) {
        let mut handlers = self.handlers.lock().await;
        // fire event!
        match (*handlers).remove(&sid).unwrap() {
            Handler::ReqRR(sender) => sender.send(Ok(input)).unwrap(),
            Handler::ResRR(c) => unreachable!(),
            Handler::ReqRS(sender) => {
                if flag & frame::FLAG_NEXT != 0 {
                    sender.send(Ok(input)).unwrap();
                }
                if flag & frame::FLAG_COMPLETE == 0 {
                    (*handlers).insert(sid, Handler::ReqRS(sender));
                }
            }
            Handler::ReqRC(sender) => {
                // TODO: support channel
                if flag & frame::FLAG_NEXT != 0 {
                    sender.send(Ok(input)).unwrap();
                }
                if flag & frame::FLAG_COMPLETE == 0 {
                    (*handlers).insert(sid, Handler::ReqRC(sender));
                }
            }
        };
    }

    #[inline]
    fn on_setup(
        &self,
        acceptor: &Acceptor,
        sid: u32,
        flag: u16,
        setup: SetupPayload,
    ) -> Result<(), Box<dyn Error>> {
        match acceptor {
            Acceptor::Simple(gen) => {
                self.responder.set(gen());
                Ok(())
            }
            Acceptor::Generate(gen) => match gen(setup, Box::new(self.clone())) {
                Ok(it) => {
                    self.responder.set(it);
                    Ok(())
                }
                Err(e) => Err(e),
            },
            Acceptor::Empty() => {
                self.responder.set(Box::new(EmptyRSocket));
                Ok(())
            }
        }
    }

    #[inline]
    async fn on_fire_and_forget(&self, sid: u32, flag: u16, input: Payload) {
        self.responder.clone().fire_and_forget(input).await
    }

    #[inline]
    async fn on_request_response(&self, sid: u32, _flag: u16, input: Payload) {
        let responder = self.responder.clone();
        let canceller = self.canceller.clone();
        let tx = self.tx.clone();

        let counter = Counter::new(2);
        self.register_handler(sid, Handler::ResRR(counter.clone()))
            .await;

        tokio::spawn(async move {
            // TODO: use future select
            let result = responder.request_response(input).await;
            if counter.count_down() == 0 {
                // cancelled
                return;
            }

            // async remove canceller
            canceller.send(sid).unwrap();

            let sending = match result {
                Ok(it) => {
                    let (d, m) = it.split();
                    let mut bu =
                        frame::Payload::builder(sid, frame::FLAG_NEXT | frame::FLAG_COMPLETE);
                    if let Some(b) = d {
                        bu = bu.set_data(b);
                    }
                    if let Some(b) = m {
                        bu = bu.set_metadata(b);
                    }
                    bu.build()
                }
                Err(e) => frame::Error::builder(sid, 0)
                    .set_code(errors::ERR_APPLICATION)
                    .set_data(Bytes::from("TODO: should be error details"))
                    .build(),
            };
            if let Err(e) = tx.send(sending) {
                error!("respond REQUEST_RESPONSE failed: {}", e);
            }
        });
    }

    #[inline]
    async fn on_request_stream(&self, sid: u32, flag: u16, input: Payload) {
        let responder = self.responder.clone();
        let tx = self.tx.clone();
        tokio::spawn(async move {
            // TODO: support cancel
            let mut payloads = responder.request_stream(input);
            while let Some(next) = payloads.next().await {
                let sending = match next {
                    Ok(it) => {
                        let (d, m) = it.split();
                        let mut bu = frame::Payload::builder(sid, frame::FLAG_NEXT);
                        if let Some(b) = d {
                            bu = bu.set_data(b);
                        }
                        if let Some(b) = m {
                            bu = bu.set_metadata(b);
                        }
                        bu.build()
                    }
                    Err(e) => frame::Error::builder(sid, 0)
                        .set_code(errors::ERR_APPLICATION)
                        .set_data(Bytes::from(format!("{}", e)))
                        .build(),
                };
                tx.send(sending).unwrap();
            }
            let complete = frame::Payload::builder(sid, frame::FLAG_COMPLETE).build();
            tx.send(complete).unwrap();
        });
    }

    #[inline]
    async fn on_request_channel(&self, sid: u32, flag: u16, first: Payload) {
        let responder = self.responder.clone();
        let tx = self.tx.clone();
        let (sender, receiver) = new_tx_rx::<Result<Payload, RSocketError>>();
        sender.send(Ok(first)).unwrap();
        self.register_handler(sid, Handler::ReqRC(sender)).await;
        tokio::spawn(async move {
            // respond client channel
            let mut outputs = responder.request_channel(Box::pin(receiver));
            // TODO: support custom RequestN.
            let request_n = frame::RequestN::builder(sid, 0).build();

            if let Err(e) = tx.send(request_n) {
                error!("respond REQUEST_N failed: {}", e);
            }

            while let Some(next) = outputs.next().await {
                let sending = match next {
                    Ok(v) => {
                        let (d, m) = v.split();
                        let mut bu = frame::Payload::builder(sid, frame::FLAG_NEXT);
                        if let Some(b) = d {
                            bu = bu.set_data(b);
                        }
                        if let Some(b) = m {
                            bu = bu.set_metadata(b);
                        }
                        bu.build()
                    }
                    Err(e) => frame::Error::builder(sid, 0)
                        .set_code(errors::ERR_APPLICATION)
                        .set_data(Bytes::from(format!("{}", e)))
                        .build(),
                };
                tx.send(sending).unwrap();
            }
            let complete = frame::Payload::builder(sid, frame::FLAG_COMPLETE).build();
            if let Err(e) = tx.send(complete) {
                error!("complete REQUEST_CHANNEL failed: {}", e);
            }
        });
    }

    #[inline]
    async fn on_metadata_push(&self, input: Payload) {
        self.responder.clone().metadata_push(input).await
    }

    #[inline]
    async fn on_keepalive(&self, keepalive: frame::Keepalive) {
        let tx = self.tx.clone();
        let (data, _) = keepalive.split();
        let mut sending = frame::Keepalive::builder(0, 0);
        if let Some(b) = data {
            sending = sending.set_data(b);
        }
        if let Err(e) = tx.send(sending.build()) {
            error!("respond KEEPALIVE failed: {}", e);
        }
    }
}

impl RSocket for DuplexSocket {
    fn metadata_push(&self, req: Payload) -> Mono<()> {
        let sid = self.seq.next();
        let tx = self.tx.clone();
        Box::pin(async move {
            let (_d, m) = req.split();
            let mut bu = frame::MetadataPush::builder(sid, 0);
            if let Some(b) = m {
                bu = bu.set_metadata(b);
            }
            if let Err(e) = tx.send(bu.build()) {
                error!("send metadata_push failed: {}", e);
            }
        })
    }
    fn fire_and_forget(&self, req: Payload) -> Mono<()> {
        let sid = self.seq.next();
        let tx = self.tx.clone();
        Box::pin(async move {
            let (d, m) = req.split();
            let mut bu = frame::RequestFNF::builder(sid, 0);
            if let Some(b) = d {
                bu = bu.set_data(b);
            }
            if let Some(b) = m {
                bu = bu.set_metadata(b);
            }
            if let Err(e) = tx.send(bu.build()) {
                error!("send fire_and_forget failed: {}", e);
            }
        })
    }
    fn request_response(&self, req: Payload) -> Mono<Result<Payload, RSocketError>> {
        let (tx, rx) = new_tx_rx_once::<Result<Payload, RSocketError>>();
        let sid = self.seq.next();
        let handlers = Arc::clone(&self.handlers);
        let sender = self.tx.clone();
        tokio::spawn(async move {
            {
                // register handler
                let mut map = handlers.lock().await;
                (*map).insert(sid, Handler::ReqRR(tx));
            }

            let (d, m) = req.split();
            // crate request frame
            let mut bu = frame::RequestResponse::builder(sid, 0);
            if let Some(b) = d {
                bu = bu.set_data(b);
            }
            if let Some(b) = m {
                bu = bu.set_metadata(b);
            }
            // send frame
            if let Err(e) = sender.send(bu.build()) {
                error!("send request_response failed: {}", e);
            }
        });
        Box::pin(async move {
            match rx.await {
                Ok(v) => v,
                Err(_e) => Err(RSocketError::from("request_response failed")),
            }
        })
    }

    fn request_stream(&self, input: Payload) -> Flux<Result<Payload, RSocketError>> {
        let sid = self.seq.next();
        let tx = self.tx.clone();
        // register handler
        let (sender, receiver) = new_tx_rx::<Result<Payload, RSocketError>>();
        let handlers = Arc::clone(&self.handlers);
        tokio::spawn(async move {
            {
                let mut map = handlers.lock().await;
                (*map).insert(sid, Handler::ReqRS(sender));
            }
            let (d, m) = input.split();
            // crate stream frame
            let mut bu = frame::RequestStream::builder(sid, 0);
            if let Some(b) = d {
                bu = bu.set_data(b);
            }
            if let Some(b) = m {
                bu = bu.set_metadata(b);
            }
            if let Err(e) = tx.send(bu.build()) {
                error!("send request_stream failed: {}", e);
            }
        });
        Box::pin(receiver)
    }

    fn request_channel(
        &self,
        mut reqs: Flux<Result<Payload, RSocketError>>,
    ) -> Flux<Result<Payload, RSocketError>> {
        let sid = self.seq.next();
        let tx = self.tx.clone();
        // register handler
        let (sender, receiver) = new_tx_rx::<Result<Payload, RSocketError>>();
        let handlers = Arc::clone(&self.handlers);
        tokio::spawn(async move {
            {
                let mut map = handlers.lock().await;
                (*map).insert(sid, Handler::ReqRC(sender));
            }
            let mut first = true;
            while let Some(next) = reqs.next().await {
                let sending = match next {
                    Ok(it) => {
                        let (d, m) = it.split();
                        if first {
                            first = false;
                            let mut bu = frame::RequestChannel::builder(sid, frame::FLAG_NEXT);
                            if let Some(b) = d {
                                bu = bu.set_data(b);
                            }
                            if let Some(b) = m {
                                bu = bu.set_metadata(b);
                            }
                            bu.build()
                        } else {
                            let mut bu = frame::Payload::builder(sid, frame::FLAG_NEXT);
                            if let Some(b) = d {
                                bu = bu.set_data(b);
                            }
                            if let Some(b) = m {
                                bu = bu.set_metadata(b);
                            }
                            bu.build()
                        }
                    }
                    Err(e) => frame::Error::builder(sid, 0)
                        .set_code(errors::ERR_APPLICATION)
                        .set_data(Bytes::from(format!("{}", e)))
                        .build(),
                };
                if let Err(e) = tx.send(sending) {
                    error!("send REQUEST_CHANNEL failed: {}", e);
                }
            }
            let sending = frame::Payload::builder(sid, frame::FLAG_COMPLETE).build();
            if let Err(e) = tx.send(sending) {
                error!("complete REQUEST_CHANNEL failed: {}", e);
            }
        });
        Box::pin(receiver)
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
    fn metadata_push(&self, req: Payload) -> Mono<()> {
        let inner = self.inner.read().unwrap();
        (*inner).metadata_push(req)
    }

    fn fire_and_forget(&self, req: Payload) -> Mono<()> {
        let inner = self.inner.read().unwrap();
        (*inner).fire_and_forget(req)
    }

    fn request_response(&self, req: Payload) -> Mono<Result<Payload, RSocketError>> {
        let inner = self.inner.read().unwrap();
        (*inner).request_response(req)
    }

    fn request_stream(&self, req: Payload) -> Flux<Result<Payload, RSocketError>> {
        let inner = self.inner.read().unwrap();
        (*inner).request_stream(req)
    }
    fn request_channel(
        &self,
        reqs: Flux<Result<Payload, RSocketError>>,
    ) -> Flux<Result<Payload, RSocketError>> {
        let inner = self.inner.read().unwrap();
        (*inner).request_channel(reqs)
    }
}
