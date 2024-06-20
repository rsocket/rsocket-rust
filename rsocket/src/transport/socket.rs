use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, Weak};

use async_stream::stream;
use async_trait::async_trait;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use dashmap::{mapref::entry::Entry, DashMap};
use futures::future::{AbortHandle, Abortable};
use futures::{Sink, SinkExt, Stream, StreamExt};
use tokio::sync::{mpsc, oneshot, RwLock};

use super::fragmentation::{Joiner, Splitter};
use super::misc::{debug_frame, Counter, StreamID};
use super::spi::*;
use crate::error::{self, RSocketError};
use crate::frame::{self, Body, Frame};
use crate::payload::{Payload, SetupPayload};
use crate::spi::{Flux, RSocket, ServerResponder};
use crate::utils::EmptyRSocket;
use crate::{runtime, Result};

struct DuplexSocketInner {
    seq: StreamID,
    responder: Responder,
    tx: mpsc::UnboundedSender<Frame>,
    handlers: DashMap<u32, Handler>,
    splitter: Option<Splitter>,
    joiners: DashMap<u32, Joiner>,
    /// AbortHandles for Response futures/streams
    abort_handles: Arc<DashMap<u32, AbortHandle>>,
}

#[derive(Clone)]
pub(crate) struct ClientRequester {
    inner: Arc<DuplexSocketInner>,
}

#[derive(Clone)]
pub(crate) struct ServerRequester {
    inner: Weak<DuplexSocketInner>,
}

pub(crate) struct DuplexSocket {
    inner: Arc<DuplexSocketInner>,
}

#[derive(Clone)]
struct Responder {
    inner: Arc<RwLock<Box<dyn RSocket>>>,
}

#[derive(Debug)]
enum Handler {
    ReqRR(oneshot::Sender<Result<Option<Payload>>>),
    ReqRS(mpsc::Sender<Result<Payload>>),
    ReqRC(mpsc::Sender<Result<Payload>>),
}

struct Cancel {}

impl DuplexSocketInner {
    fn new(
        first_stream_id: u32,
        tx: mpsc::UnboundedSender<Frame>,
        splitter: Option<Splitter>,
    ) -> Self {
        let (canceller_tx, canceller_rx) = mpsc::channel::<u32>(32);
        let this = Self {
            seq: StreamID::from(first_stream_id),
            tx,
            responder: Responder::new(),
            handlers: DashMap::new(),
            joiners: DashMap::new(),
            splitter,
            abort_handles: Arc::new(DashMap::new()),
        };
        this
    }
}

impl DuplexSocket {
    pub(crate) fn new(
        first_stream_id: u32,
        tx: mpsc::UnboundedSender<Frame>,
        splitter: Option<Splitter>,
    ) -> DuplexSocket {
        DuplexSocket {
            inner: Arc::new(DuplexSocketInner::new(first_stream_id, tx, splitter)),
        }
    }

    pub(crate) async fn setup(&mut self, setup: SetupPayload) -> Result<()> {
        let mut bu = frame::Setup::builder(0, 0);
        if let Some(s) = setup.data_mime_type() {
            bu = bu.set_mime_data(s);
        }
        if let Some(s) = setup.metadata_mime_type() {
            bu = bu.set_mime_metadata(s);
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
        self.inner.tx.send(bu.build()).map_err(|e| e.into())
    }

    #[inline]
    fn register_handler(&self, sid: u32, handler: Handler) {
        self.inner.handlers.insert(sid, handler);
    }

    pub(crate) async fn dispatch(
        &mut self,
        frame: Frame,
        acceptor: Option<&ServerResponder>,
    ) -> Result<()> {
        if let Some(frame) = self.join_frame(frame).await {
            self.process_once(frame, acceptor).await;
        }
        Ok(())
    }

    #[inline]
    async fn process_once(&mut self, msg: Frame, acceptor: Option<&ServerResponder>) {
        let sid = msg.get_stream_id();
        let flag = msg.get_flag();
        debug_frame(false, &msg);
        match msg.get_body() {
            Body::Setup(v) => {
                if let Err(e) = self
                    .on_setup(acceptor, sid, flag, SetupPayload::from(v))
                    .await
                {
                    let errmsg = format!("{}", e);
                    let sending = frame::Error::builder(0, 0)
                        .set_code(error::ERR_REJECT_SETUP)
                        .set_data(Bytes::from(errmsg))
                        .build();
                    if self.inner.tx.send(sending).is_err() {
                        error!("Reject setup failed");
                    }
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
                self.on_fire_and_forget(sid, input).await;
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
                if flag & Frame::FLAG_RESPOND != 0 {
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

    #[inline]
    async fn join_frame(&self, input: Frame) -> Option<Frame> {
        let (is_follow, is_payload) = input.is_followable_or_payload();
        if !is_follow {
            return Some(input);
        }
        let sid = input.get_stream_id();
        if input.get_flag() & Frame::FLAG_FOLLOW != 0 {
            // TODO: check conflict
            self.inner
                .joiners
                .entry(sid)
                .or_insert_with(Joiner::new)
                .push(input);
            return None;
        }

        if !is_payload {
            return Some(input);
        }

        match self.inner.joiners.remove(&sid) {
            None => Some(input),
            Some((_, mut joiner)) => {
                joiner.push(input);
                let flag = joiner.get_flag();
                let first = joiner.first();
                match &first.body {
                    frame::Body::RequestResponse(_) => {
                        let pa: Payload = joiner.into();
                        let result = frame::RequestResponse::builder(sid, flag)
                            .set_all(pa.split())
                            .build();
                        Some(result)
                    }
                    frame::Body::RequestStream(b) => {
                        let n = b.get_initial_request_n();
                        let pa: Payload = joiner.into();
                        let result = frame::RequestStream::builder(sid, flag)
                            .set_initial_request_n(n)
                            .set_all(pa.split())
                            .build();
                        Some(result)
                    }
                    frame::Body::RequestFNF(_) => {
                        let pa: Payload = joiner.into();
                        let result = frame::RequestFNF::builder(sid, flag)
                            .set_all(pa.split())
                            .build();
                        Some(result)
                    }
                    frame::Body::RequestChannel(b) => {
                        let n = b.get_initial_request_n();
                        let pa: Payload = joiner.into();
                        let result = frame::RequestChannel::builder(sid, flag)
                            .set_initial_request_n(n)
                            .set_all(pa.split())
                            .build();
                        Some(result)
                    }
                    frame::Body::Payload(b) => {
                        let pa: Payload = joiner.into();
                        let result = frame::Payload::builder(sid, flag)
                            .set_all(pa.split())
                            .build();
                        Some(result)
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    #[inline]
    async fn on_error(&mut self, sid: u32, flag: u16, input: frame::Error) {
        self.inner.joiners.remove(&sid);
        // pick handler
        if let Some((_, handler)) = self.inner.handlers.remove(&sid) {
            let desc = input
                .get_data_utf8()
                .map(|it| it.to_string())
                .unwrap_or_default();
            let e = RSocketError::must_new_from_code(input.get_code(), desc);
            match handler {
                Handler::ReqRR(tx) => {
                    if tx.send(Err(e.into())).is_err() {
                        error!("respond with error for REQUEST_RESPONSE failed!");
                    }
                }
                Handler::ReqRS(tx) => {
                    if (tx.send(Err(e.into())).await).is_err() {
                        error!("respond with error for REQUEST_STREAM failed!");
                    };
                }
                Handler::ReqRC(tx) => {
                    if (tx.send(Err(e.into())).await).is_err() {
                        error!("respond with error for REQUEST_CHANNEL failed!");
                    }
                }
            }
        }
    }

    #[inline]
    async fn on_cancel(&mut self, sid: u32, _flag: u16) {
        if let Some((sid, abort_handle)) = self.inner.abort_handles.remove(&sid) {
            abort_handle.abort();
        }
        self.inner.joiners.remove(&sid);
        if let Some((_, handler)) = self.inner.handlers.remove(&sid) {
            let e: Result<_> =
                Err(RSocketError::RequestCancelled("request has been cancelled".into()).into());
            match handler {
                Handler::ReqRR(sender) => {
                    info!("REQUEST_RESPONSE {} cancelled!", sid);
                    if sender.send(e).is_err() {
                        error!("notify cancel for REQUEST_RESPONSE failed: sid={}", sid);
                    }
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
    async fn on_payload(&mut self, sid: u32, flag: u16, input: Payload) {
        match self.inner.handlers.entry(sid) {
            Entry::Occupied(o) => {
                match o.get() {
                    Handler::ReqRR(_) => match o.remove() {
                        Handler::ReqRR(sender) => {
                            if flag & Frame::FLAG_NEXT != 0 {
                                if sender.send(Ok(Some(input))).is_err() {
                                    error!("response successful payload for REQUEST_RESPONSE failed: sid={}",sid);
                                }
                            } else if sender.send(Ok(None)).is_err() {
                                error!("response successful payload for REQUEST_RESPONSE failed: sid={}",sid);
                            }
                        }
                        _ => unreachable!(),
                    },
                    Handler::ReqRS(sender) => {
                        if flag & Frame::FLAG_NEXT != 0 {
                            if sender.is_closed() {
                                self.send_cancel_frame(sid);
                            } else if let Err(e) = sender.send(Ok(input)).await {
                                error!(
                                    "response successful payload for REQUEST_STREAM failed: sid={}",
                                    sid
                                );
                                self.send_cancel_frame(sid);
                            }
                        }
                        if flag & Frame::FLAG_COMPLETE != 0 {
                            o.remove();
                        }
                    }
                    Handler::ReqRC(sender) => {
                        // TODO: support channel
                        if flag & Frame::FLAG_NEXT != 0 {
                            if sender.is_closed() {
                                self.send_cancel_frame(sid);
                            } else if (sender.clone().send(Ok(input)).await).is_err() {
                                error!("response successful payload for REQUEST_CHANNEL failed: sid={}",sid);
                                self.send_cancel_frame(sid);
                            }
                        }
                        if flag & Frame::FLAG_COMPLETE != 0 {
                            o.remove();
                        }
                    }
                }
            }
            Entry::Vacant(_) => warn!("invalid payload id {}: no such request!", sid),
        }
    }

    #[inline]
    fn send_cancel_frame(&self, sid: u32) {
        let cancel_frame = frame::Cancel::builder(sid, Frame::FLAG_COMPLETE).build();
        if let Err(e) = self.inner.tx.send(cancel_frame) {
            error!("Sending CANCEL frame failed: sid={}, reason: {}", sid, e);
        }
    }

    pub(crate) async fn bind_responder(&self, responder: Box<dyn RSocket>) {
        self.inner.responder.set(responder).await;
    }

    #[inline]
    async fn on_setup(
        &self,
        acceptor: Option<&ServerResponder>,
        sid: u32,
        flag: u16,
        setup: SetupPayload,
    ) -> Result<()> {
        match acceptor {
            None => {
                self.inner.responder.set(Box::new(EmptyRSocket)).await;
                Ok(())
            }
            Some(gen) => match gen(setup, Box::new(self.server_requester())) {
                Ok(it) => {
                    self.inner.responder.set(it).await;
                    Ok(())
                }
                Err(e) => Err(e),
            },
        }
    }

    #[inline]
    async fn on_fire_and_forget(&mut self, sid: u32, input: Payload) {
        // TODO: Spawning a task here in case responer call goes  pending, which would hold
        // up the entire dispatch loop
        if let Err(e) = self.inner.responder.fire_and_forget(input).await {
            error!("respond fire_and_forget failed: {:?}", e);
        }
    }

    #[inline]
    async fn on_request_response(&mut self, sid: u32, _flag: u16, input: Payload) {
        let responder = self.inner.responder.clone();

        let mut tx = self.inner.tx.clone();
        let splitter = self.inner.splitter.clone();    
        let (abort_handle, abort_registration) = AbortHandle::new_pair();       
        let abort_handles = self.inner.abort_handles.clone();
        runtime::spawn(async move {
                       
            abort_handles.insert(sid, abort_handle);
            let result= Abortable::new(responder.request_response(input), abort_registration).await;
            abort_handles.remove(&sid);

            // Abort for futures adds an extra result wrapper, so unwrap that and continue
            let Ok(result) = result else {
                // cancelled
                return;
            };

            match result {
                Ok(Some(res)) => {
                    DuplexSocketInner::try_send_payload(
                        &splitter,
                        &mut tx,
                        sid,
                        res,
                        Frame::FLAG_NEXT | Frame::FLAG_COMPLETE,
                    )
                    .await;
                }
                Ok(None) => {
                    DuplexSocketInner::try_send_complete(&mut tx, sid, Frame::FLAG_COMPLETE).await;
                }
                Err(e) => {
                    let sending = frame::Error::builder(sid, 0)
                        .set_code(error::ERR_APPLICATION)
                        .set_data(Bytes::from(e.to_string()))
                        .build();
                    if let Err(e) = tx.send(sending) {
                        error!("respond REQUEST_RESPONSE failed: {}", e);
                    }
                }
            };
        });
    }

    #[inline]
    async fn on_request_stream(&self, sid: u32, flag: u16, input: Payload) {
        let responder = self.inner.responder.clone();
        let mut tx = self.inner.tx.clone();
        let splitter = self.inner.splitter.clone();
        let abort_handles = self.inner.abort_handles.clone();
        runtime::spawn(async move {
            let (abort_handle, abort_registration) = AbortHandle::new_pair();
            abort_handles.insert(sid, abort_handle);
            let mut payloads = Abortable::new(responder.request_stream(input), abort_registration);
            while let Some(next) = payloads.next().await {
                match next {
                    Ok(it) => {
                        DuplexSocketInner::try_send_payload(
                            &splitter,
                            &mut tx,
                            sid,
                            it,
                            Frame::FLAG_NEXT,
                        )
                        .await;
                    }
                    Err(e) => {
                        let sending = frame::Error::builder(sid, 0)
                            .set_code(error::ERR_APPLICATION)
                            .set_data(Bytes::from(format!("{}", e)))
                            .build();
                        tx.send(sending).expect("Send stream response failed");
                    }
                };
            }
            abort_handles.remove(&sid);
            let complete = frame::Payload::builder(sid, Frame::FLAG_COMPLETE).build();
            tx.send(complete)
                .expect("Send stream complete response failed");
        });
    }

    #[inline]
    async fn on_request_channel(&self, sid: u32, flag: u16, first: Payload) {
        let responder = self.inner.responder.clone();
        let tx = self.inner.tx.clone();
        let (sender, mut receiver) = mpsc::channel::<Result<Payload>>(32);
        sender.send(Ok(first)).await.expect("Send failed!");
        self.register_handler(sid, Handler::ReqRC(sender));
        let abort_handles = self.inner.abort_handles.clone();
        runtime::spawn(async move {
            // respond client channel
            let outputs = responder.request_channel(Box::pin(stream! {
                while let Some(it) = receiver.recv().await{
                    yield it;
                }
            }));
            let (abort_handle, abort_registration) = AbortHandle::new_pair();
            abort_handles.insert(sid, abort_handle);
            let mut outputs = Abortable::new(outputs, abort_registration);

            // TODO: support custom RequestN.
            let request_n = frame::RequestN::builder(sid, 0).build();

            if let Err(e) = tx.send(request_n) {
                error!("respond REQUEST_N failed: {}", e);
            }

            while let Some(next) = outputs.next().await {
                let sending = match next {
                    Ok(payload) => {
                        let (data, metadata) = payload.split();
                        let mut bu = frame::Payload::builder(sid, Frame::FLAG_NEXT);
                        if let Some(b) = data {
                            bu = bu.set_data(b);
                        }
                        if let Some(b) = metadata {
                            bu = bu.set_metadata(b);
                        }
                        bu.build()
                    }
                    Err(e) => frame::Error::builder(sid, 0)
                        .set_code(error::ERR_APPLICATION)
                        .set_data(Bytes::from(format!("{}", e)))
                        .build(),
                };
                tx.send(sending).expect("Send failed!");
            }
            abort_handles.remove(&sid);
            let complete = frame::Payload::builder(sid, Frame::FLAG_COMPLETE).build();
            if let Err(e) = tx.send(complete) {
                error!("complete REQUEST_CHANNEL failed: {}", e);
            }
        });
    }

    #[inline]
    async fn on_metadata_push(&mut self, input: Payload) {
        // TODO: Spawning a task here in case responer call goes  pending, which would hold
        // up the entire dispatch loop
        if let Err(e) = self.inner.responder.metadata_push(input).await {
            error!("response metadata_push failed: {:?}", e);
        }
    }

    #[inline]
    async fn on_keepalive(&mut self, keepalive: frame::Keepalive) {
        let (data, _) = keepalive.split();
        let mut sending = frame::Keepalive::builder(0, 0);
        if let Some(b) = data {
            sending = sending.set_data(b);
        }
        if let Err(e) = self.inner.tx.send(sending.build()) {
            error!("respond KEEPALIVE failed: {}", e);
        }
    }

    pub(crate) fn client_requester(&self) -> ClientRequester {
        ClientRequester {
            inner: self.inner.clone(),
        }
    }

    pub(crate) fn server_requester(&self) -> ServerRequester {
        ServerRequester {
            inner: Arc::downgrade(&self.inner),
        }
    }
}

// These are the immplementation functions for the requesters below
impl DuplexSocketInner {
    async fn metadata_push(&self, req: Payload) -> Result<()> {
        let sid = self.seq.next();
        let tx = self.tx.clone();
        let (_d, m) = req.split();
        let mut bu = frame::MetadataPush::builder(sid, 0);
        if let Some(b) = m {
            bu = bu.set_metadata(b);
        }
        tx.send(bu.build())?;
        Ok(())
    }

    async fn fire_and_forget(&self, req: Payload) -> Result<()> {
        let sid = self.seq.next();
        let tx = self.tx.clone();
        let splitter = self.splitter.clone();

        match splitter {
            Some(sp) => {
                let mut cuts: usize = 0;
                let mut prev: Option<Payload> = None;
                for next in sp.cut(req, 0) {
                    if let Some(cur) = prev.take() {
                        let sending = if cuts == 1 {
                            // make first frame as request_fnf.
                            frame::RequestFNF::builder(sid, Frame::FLAG_FOLLOW)
                                .set_all(cur.split())
                                .build()
                        } else {
                            // make other frames as payload.
                            frame::Payload::builder(sid, Frame::FLAG_FOLLOW)
                                .set_all(cur.split())
                                .build()
                        };
                        // send frame
                        tx.send(sending)?;
                    }
                    prev = Some(next);
                    cuts += 1;
                }

                let sending = if cuts == 0 {
                    frame::RequestFNF::builder(sid, 0).build()
                } else if cuts == 1 {
                    frame::RequestFNF::builder(sid, 0)
                        .set_all(prev.unwrap().split())
                        .build()
                } else {
                    frame::Payload::builder(sid, 0)
                        .set_all(prev.unwrap().split())
                        .build()
                };
                // send frame
                tx.send(sending)?;
            }
            None => {
                let sending = frame::RequestFNF::builder(sid, 0)
                    .set_all(req.split())
                    .build();
                tx.send(sending)?;
            }
        }
        Ok(())
    }

    async fn request_response(&self, req: Payload) -> Result<Option<Payload>> {
        let (tx, rx) = oneshot::channel::<Result<Option<Payload>>>();
        let sid = self.seq.next();
        let sender = self.tx.clone();
        let splitter = self.splitter.clone();

        // Register handler
        self.handlers.insert(sid, Handler::ReqRR(tx));

        runtime::spawn(async move {
            match splitter {
                Some(sp) => {
                    let mut cuts: usize = 0;
                    let mut prev: Option<Payload> = None;
                    for next in sp.cut(req, 0) {
                        if let Some(cur) = prev.take() {
                            let sending = if cuts == 1 {
                                // make first frame as request_response.
                                frame::RequestResponse::builder(sid, Frame::FLAG_FOLLOW)
                                    .set_all(cur.split())
                                    .build()
                            } else {
                                // make other frames as payload.
                                frame::Payload::builder(sid, Frame::FLAG_FOLLOW)
                                    .set_all(cur.split())
                                    .build()
                            };
                            // send frame
                            if let Err(e) = sender.send(sending) {
                                error!("send request_response failed: {}", e);
                                return;
                            }
                        }
                        prev = Some(next);
                        cuts += 1;
                    }

                    let sending = if cuts == 0 {
                        frame::RequestResponse::builder(sid, 0).build()
                    } else if cuts == 1 {
                        frame::RequestResponse::builder(sid, 0)
                            .set_all(prev.unwrap().split())
                            .build()
                    } else {
                        frame::Payload::builder(sid, 0)
                            .set_all(prev.unwrap().split())
                            .build()
                    };
                    // send frame
                    if let Err(e) = sender.send(sending) {
                        error!("send request_response failed: {}", e);
                    }
                }
                None => {
                    // crate request frame
                    let sending = frame::RequestResponse::builder(sid, 0)
                        .set_all(req.split())
                        .build();
                    // send frame
                    if let Err(e) = sender.send(sending) {
                        error!("send request_response failed: {}", e);
                    }
                }
            }
        });
        match rx.await {
            Ok(v) => v,
            Err(_e) => Err(RSocketError::WithDescription("request_response failed".into()).into()),
        }
    }

    fn request_stream(&self, input: Payload) -> Flux<Result<Payload>> {
        let sid = self.seq.next();
        let tx = self.tx.clone();
        // register handler
        let (sender, mut receiver) = mpsc::channel::<Result<Payload>>(32);
        self.handlers.insert(sid, Handler::ReqRS(sender));
        let splitter = self.splitter.clone();
        runtime::spawn(async move {
            match splitter {
                Some(sp) => {
                    let mut cuts: usize = 0;
                    let mut prev: Option<Payload> = None;
                    // skip 4 bytes. (initial_request_n is u32)
                    for next in sp.cut(input, 4) {
                        if let Some(cur) = prev.take() {
                            let sending: Frame = if cuts == 1 {
                                // make first frame as request_stream.
                                frame::RequestStream::builder(sid, Frame::FLAG_FOLLOW)
                                    .set_all(cur.split())
                                    .build()
                            } else {
                                // make other frames as payload.
                                frame::Payload::builder(sid, Frame::FLAG_FOLLOW)
                                    .set_all(cur.split())
                                    .build()
                            };
                            // send frame
                            if let Err(e) = tx.send(sending) {
                                error!("send request_stream failed: {}", e);
                                return;
                            }
                        }
                        prev = Some(next);
                        cuts += 1;
                    }

                    let sending = if cuts == 0 {
                        frame::RequestStream::builder(sid, 0).build()
                    } else if cuts == 1 {
                        frame::RequestStream::builder(sid, 0)
                            .set_all(prev.unwrap().split())
                            .build()
                    } else {
                        frame::Payload::builder(sid, 0)
                            .set_all(prev.unwrap().split())
                            .build()
                    };
                    // send frame
                    if let Err(e) = tx.send(sending) {
                        error!("send request_stream failed: {}", e);
                    }
                }
                None => {
                    let sending = frame::RequestStream::builder(sid, 0)
                        .set_all(input.split())
                        .build();
                    if let Err(e) = tx.send(sending) {
                        error!("send request_stream failed: {}", e);
                    }
                }
            }
        });
        Box::pin(stream! {
            while let Some(it) = receiver.recv().await{
                yield it;
            }
        })
    }

    fn request_channel(&self, mut reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        let sid = self.seq.next();
        let mut tx = self.tx.clone();

        let (sender, mut receiver) = mpsc::channel::<Result<Payload>>(32);
        // register handler
        self.handlers.insert(sid, Handler::ReqRC(sender));
        let splitter = self.splitter.clone();
        runtime::spawn(async move {
            let mut first = true;
            while let Some(next) = reqs.next().await {
                match next {
                    Ok(it) => {
                        if first {
                            first = false;
                            Self::try_send_channel(&splitter, &mut tx, sid, it, Frame::FLAG_NEXT)
                                .await
                        } else {
                            Self::try_send_payload(&splitter, &mut tx, sid, it, Frame::FLAG_NEXT)
                                .await
                        }
                    }
                    Err(e) => {
                        let sending = frame::Error::builder(sid, 0)
                            .set_code(error::ERR_APPLICATION)
                            .set_data(Bytes::from(format!("{}", e)))
                            .build();
                        if let Err(e) = tx.send(sending) {
                            error!("send REQUEST_CHANNEL failed: {}", e);
                        }
                    }
                };
            }
            let sending = frame::Payload::builder(sid, Frame::FLAG_COMPLETE).build();
            if let Err(e) = tx.send(sending) {
                error!("complete REQUEST_CHANNEL failed: {}", e);
            }
        });
        Box::pin(stream! {
            while let Some(it) = receiver.recv().await{
                yield it;
            }
        })
    }

    #[inline]
    async fn try_send_channel(
        splitter: &Option<Splitter>,
        tx: &mut mpsc::UnboundedSender<Frame>,
        sid: u32,
        res: Payload,
        flag: u16,
    ) {
        // TODO
        match splitter {
            Some(sp) => {
                let mut cuts: usize = 0;
                let mut prev: Option<Payload> = None;
                for next in sp.cut(res, 4) {
                    if let Some(cur) = prev.take() {
                        let sending = if cuts == 1 {
                            frame::RequestChannel::builder(sid, flag | Frame::FLAG_FOLLOW)
                                .set_all(cur.split())
                                .build()
                        } else {
                            frame::Payload::builder(sid, Frame::FLAG_FOLLOW)
                                .set_all(cur.split())
                                .build()
                        };
                        // send frame
                        if let Err(e) = tx.send(sending) {
                            error!("send request_channel failed: {}", e);
                            return;
                        }
                    }
                    prev = Some(next);
                    cuts += 1;
                }

                let sending = if cuts == 0 {
                    frame::RequestChannel::builder(sid, flag).build()
                } else if cuts == 1 {
                    frame::RequestChannel::builder(sid, flag)
                        .set_all(prev.unwrap().split())
                        .build()
                } else {
                    frame::Payload::builder(sid, 0)
                        .set_all(prev.unwrap().split())
                        .build()
                };
                // send frame
                if let Err(e) = tx.send(sending) {
                    error!("send request_channel failed: {}", e);
                }
            }
            None => {
                let sending = frame::RequestChannel::builder(sid, flag)
                    .set_all(res.split())
                    .build();
                if let Err(e) = tx.send(sending) {
                    error!("send request_channel failed: {}", e);
                }
            }
        }
    }

    #[inline]
    async fn try_send_complete(tx: &mut mpsc::UnboundedSender<Frame>, sid: u32, flag: u16) {
        let sending = frame::Payload::builder(sid, flag).build();
        if let Err(e) = tx.send(sending) {
            error!("respond failed: {}", e);
        }
    }

    #[inline]
    async fn try_send_payload(
        splitter: &Option<Splitter>,
        tx: &mut mpsc::UnboundedSender<Frame>,
        sid: u32,
        res: Payload,
        flag: u16,
    ) {
        match splitter {
            Some(sp) => {
                let mut cuts: usize = 0;
                let mut prev: Option<Payload> = None;
                for next in sp.cut(res, 0) {
                    if let Some(cur) = prev.take() {
                        let sending = if cuts == 1 {
                            frame::Payload::builder(sid, flag | Frame::FLAG_FOLLOW)
                                .set_all(cur.split())
                                .build()
                        } else {
                            frame::Payload::builder(sid, Frame::FLAG_FOLLOW)
                                .set_all(cur.split())
                                .build()
                        };
                        // send frame
                        if let Err(e) = tx.send(sending) {
                            error!("send payload failed: {}", e);
                            return;
                        }
                    }
                    prev = Some(next);
                    cuts += 1;
                }

                let sending = if cuts == 0 {
                    frame::Payload::builder(sid, flag).build()
                } else {
                    frame::Payload::builder(sid, flag)
                        .set_all(prev.unwrap().split())
                        .build()
                };
                // send frame
                if let Err(e) = tx.send(sending) {
                    error!("send payload failed: {}", e);
                }
            }
            None => {
                let sending = frame::Payload::builder(sid, flag)
                    .set_all(res.split())
                    .build();
                if let Err(e) = tx.send(sending) {
                    error!("respond failed: {}", e);
                }
            }
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

    async fn set(&self, rs: Box<dyn RSocket>) {
        let mut w = self.inner.write().await;
        *w = rs;
    }
}

#[async_trait]
impl RSocket for Responder {
    async fn metadata_push(&self, req: Payload) -> Result<()> {
        let inner = self.inner.read().await;
        (*inner).metadata_push(req).await
    }

    async fn fire_and_forget(&self, req: Payload) -> Result<()> {
        let inner = self.inner.read().await;
        (*inner).fire_and_forget(req).await
    }

    async fn request_response(&self, req: Payload) -> Result<Option<Payload>> {
        let inner = self.inner.read().await;
        (*inner).request_response(req).await
    }

    fn request_stream(&self, req: Payload) -> Flux<Result<Payload>> {
        let inner = self.inner.clone();
        Box::pin(stream! {
            let r = inner.read().await;
            let mut results = (*r).request_stream(req);
            while let Some(next) = results.next().await {
                yield next;
            }
        })
    }

    fn request_channel(&self, reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        let inner = self.inner.clone();
        Box::pin(stream! {
            let r = inner.read().await;
            let mut results = (*r).request_channel(reqs);
            while let Some(next) = results.next().await{
                yield next;
            }
        })
    }
}

#[async_trait]
impl RSocket for ClientRequester {
    /// Metadata-Push interaction model of RSocket.
    async fn metadata_push(&self, req: Payload) -> Result<()> {
        self.inner.metadata_push(req).await
    }
    /// Fire and Forget interaction model of RSocket.
    async fn fire_and_forget(&self, req: Payload) -> Result<()> {
        self.inner.fire_and_forget(req).await
    }
    /// Request-Response interaction model of RSocket.
    async fn request_response(&self, req: Payload) -> Result<Option<Payload>> {
        self.inner.request_response(req).await
    }
    /// Request-Stream interaction model of RSocket.
    fn request_stream(&self, req: Payload) -> Flux<Result<Payload>> {
        self.inner.request_stream(req)
    }
    /// Request-Channel interaction model of RSocket.
    fn request_channel(&self, reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        self.inner.request_channel(reqs)
    }
}

// This implementation, uses the async function trait, as it needs to capture the ugpraded
// Arc, at least until it returns
#[async_trait]
impl RSocket for ServerRequester {
    /// Metadata-Push interaction model of RSocket.
    async fn metadata_push(&self, req: Payload) -> Result<()> {
        match self.inner.upgrade() {
            Some(inner) => inner.metadata_push(req).await,
            None => Err(RSocketError::ConnectionClosed("close".into()).into()),
        }
    }
    /// Fire and Forget interaction model of RSocket.
    async fn fire_and_forget(&self, req: Payload) -> Result<()> {
        match self.inner.upgrade() {
            Some(inner) => inner.fire_and_forget(req).await,
            None => Err(RSocketError::ConnectionClosed("closed".into()).into()),
        }
    }
    /// Request-Response interaction model of RSocket.
    async fn request_response(&self, req: Payload) -> Result<Option<Payload>> {
        match self.inner.upgrade() {
            Some(inner) => inner.request_response(req).await,
            None => Err(RSocketError::ConnectionClosed("closed".into()).into()),
        }
    }
    /// Request-Stream interaction model of RSocket.
    fn request_stream(&self, req: Payload) -> Flux<Result<Payload>> {
        use futures::{future, stream};

        match self.inner.upgrade() {
            Some(inner) => inner.request_stream(req),
            None => Box::pin(stream::once(future::ready(Err(
                RSocketError::ConnectionClosed("closed".into()).into(),
            )))),
        }
    }
    /// Request-Channel interaction model of RSocket.
    fn request_channel(&self, reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        use futures::{future, stream};
        match self.inner.upgrade() {
            Some(inner) => inner.request_channel(reqs),
            None => Box::pin(stream::once(future::ready(Err(
                RSocketError::ConnectionClosed("closed".into()).into(),
            )))),
        }
    }
}
