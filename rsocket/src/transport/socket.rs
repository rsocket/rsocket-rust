use super::fragmentation::{Joiner, Splitter};
use super::misc::{self, Counter, StreamID};
use super::spi::*;
use crate::error::{self, ErrorKind, RSocketError};
use crate::frame::{self, Body, Frame};
use crate::payload::{Payload, SetupPayload};
use crate::runtime::Spawner;
use crate::spi::{EmptyRSocket, Flux, Mono, RSocket};
use crate::utils::RSocketResult;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{future, Sink, SinkExt, Stream, StreamExt};
use std::collections::{hash_map::Entry, HashMap};
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
pub(crate) struct DuplexSocket<R>
where
    R: Send + Sync + Clone + Spawner + 'static,
{
    rt: R,
    seq: StreamID,
    responder: Responder,
    tx: Tx<Frame>,
    handlers: [Arc<Mutex<HashMap<u32, Handler>>>; 16],
    canceller: Tx<u32>,
    splitter: Option<Splitter>,
    joiners: Arc<Mutex<HashMap<u32, Joiner>>>,
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

impl<R> DuplexSocket<R>
where
    R: Send + Sync + Clone + Spawner + 'static,
{
    pub(crate) async fn new(
        rt: R,
        first_stream_id: u32,
        tx: Tx<Frame>,
        splitter: Option<Splitter>,
    ) -> DuplexSocket<R> {
        let rt2 = rt.clone();
        let (canceller_tx, canceller_rx) = new_tx_rx::<u32>();
        let handlers = [
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
            Arc::new(Mutex::new(HashMap::new())),
        ];
        let ds = DuplexSocket {
            rt,
            seq: StreamID::from(first_stream_id),
            tx,
            canceller: canceller_tx,
            responder: Responder::new(),
            handlers,
            joiners: Arc::new(Mutex::new(HashMap::new())),
            splitter,
        };

        let ds2 = ds.clone();
        rt2.spawn(async move {
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
        self.tx
            .unbounded_send(bu.build())
            .expect("Send setup failed");
    }

    #[inline]
    fn get_handler(&self, sid: u32) -> &Arc<Mutex<HashMap<u32, Handler>>> {
        let i = ((sid ^ (sid >> 16)) & 15) as usize;
        &self.handlers[i]
    }

    #[inline]
    async fn register_handler(&self, sid: u32, handler: Handler) {
        let h = self.get_handler(sid);
        let mut handlers = h.lock().await;
        (*handlers).insert(sid, handler);
    }

    pub(crate) async fn loop_canceller(&self, mut rx: Rx<u32>) {
        while let Some(sid) = rx.next().await {
            let mut handlers = self.get_handler(sid).lock().await;
            (*handlers).remove(&sid);
        }
    }

    #[inline]
    async fn process_once(&self, msg: Frame, acceptor: &Option<Acceptor>) {
        let sid = msg.get_stream_id();
        let flag = msg.get_flag();
        misc::debug_frame(false, &msg);
        match msg.get_body() {
            Body::Setup(v) => {
                if let Err(e) = self.on_setup(acceptor, sid, flag, SetupPayload::from(v)) {
                    let errmsg = format!("{}", e);
                    let sending = frame::Error::builder(0, 0)
                        .set_code(error::ERR_REJECT_SETUP)
                        .set_data(Bytes::from(errmsg))
                        .build();
                    self.tx
                        .unbounded_send(sending)
                        .expect("Reject setup failed");
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

    pub(crate) async fn event_loop(&self, acceptor: Option<Acceptor>, mut rx: Rx<Frame>) {
        while let Some(next) = rx.next().await {
            if let Some(f) = self.join_frame(next).await {
                self.process_once(f, &acceptor).await
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
        let mut joiners = self.joiners.lock().await;

        if input.get_flag() & frame::FLAG_FOLLOW != 0 {
            // TODO: check conflict
            (*joiners)
                .entry(sid)
                .or_insert_with(Joiner::new)
                .push(input);
            return None;
        }

        if !is_payload {
            return Some(input);
        }

        match (*joiners).remove(&sid) {
            None => Some(input),
            Some(mut joiner) => {
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
    async fn on_error(&self, sid: u32, flag: u16, input: frame::Error) {
        {
            let mut joiners = self.joiners.lock().await;
            (*joiners).remove(&sid);
        }
        // pick handler
        let mut handlers = self.get_handler(sid).lock().await;
        if let Some(handler) = (*handlers).remove(&sid) {
            let kind =
                ErrorKind::Internal(input.get_code(), input.get_data_utf8().unwrap().to_owned());
            let e = Err(RSocketError::from(kind));
            match handler {
                Handler::ReqRR(tx) => tx.send(e).expect("Send RR failed"),
                Handler::ResRR(_) => unreachable!(),
                Handler::ReqRS(tx) => tx.unbounded_send(e).expect("Send RS failed"),
                Handler::ReqRC(tx) => tx.unbounded_send(e).expect("Send RC failed"),
            }
        }
    }

    #[inline]
    async fn on_cancel(&self, sid: u32, _flag: u16) {
        {
            let mut joiners = self.joiners.lock().await;
            (*joiners).remove(&sid);
        }
        let mut handlers = self.get_handler(sid).lock().await;
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
        let mut handlers = self.get_handler(sid).lock().await;
        // fire event!
        match (*handlers).entry(sid) {
            Entry::Occupied(o) => {
                match o.get() {
                    Handler::ReqRR(_) => match o.remove() {
                        Handler::ReqRR(sender) => {
                            sender.send(Ok(input)).unwrap();
                        }
                        _ => unreachable!(),
                    },
                    Handler::ResRR(c) => unreachable!(),
                    Handler::ReqRS(sender) => {
                        if flag & frame::FLAG_NEXT != 0 {
                            sender
                                .unbounded_send(Ok(input))
                                .expect("Send payload response failed.");
                        }
                        if flag & frame::FLAG_COMPLETE != 0 {
                            o.remove();
                        }
                    }
                    Handler::ReqRC(sender) => {
                        // TODO: support channel
                        if flag & frame::FLAG_NEXT != 0 {
                            sender
                                .unbounded_send(Ok(input))
                                .expect("Send payload response failed");
                        }
                        if flag & frame::FLAG_COMPLETE != 0 {
                            o.remove();
                        }
                    }
                }
            }
            Entry::Vacant(v) => warn!("invalid payload id {}: no such request!", sid),
        }
    }

    #[inline]
    fn on_setup(
        &self,
        acceptor: &Option<Acceptor>,
        sid: u32,
        flag: u16,
        setup: SetupPayload,
    ) -> Result<(), Box<dyn Error>> {
        match acceptor {
            None => {
                self.responder.set(Box::new(EmptyRSocket));
                Ok(())
            }
            Some(Acceptor::Simple(gen)) => {
                self.responder.set(gen());
                Ok(())
            }
            Some(Acceptor::Generate(gen)) => match gen(setup, Box::new(self.clone())) {
                Ok(it) => {
                    self.responder.set(it);
                    Ok(())
                }
                Err(e) => Err(e),
            },
        }
    }

    #[inline]
    async fn on_fire_and_forget(&self, sid: u32, input: Payload) {
        self.responder.clone().fire_and_forget(input).await
    }

    #[inline]
    async fn on_request_response(&self, sid: u32, _flag: u16, input: Payload) {
        let responder = self.responder.clone();
        let canceller = self.canceller.clone();
        let tx = self.tx.clone();
        let splitter = self.splitter.clone();
        let counter = Counter::new(2);
        self.register_handler(sid, Handler::ResRR(counter.clone()))
            .await;
        self.rt.spawn(async move {
            // TODO: use future select
            let result = responder.request_response(input).await;
            if counter.count_down() == 0 {
                // cancelled
                return;
            }

            // async remove canceller
            canceller
                .unbounded_send(sid)
                .expect("Send canceller failed");

            match result {
                Ok(res) => {
                    Self::try_send_payload(
                        &splitter,
                        &tx,
                        sid,
                        res,
                        frame::FLAG_NEXT | frame::FLAG_COMPLETE,
                    );
                }
                Err(e) => {
                    let sending = frame::Error::builder(sid, 0)
                        .set_code(error::ERR_APPLICATION)
                        .set_data(Bytes::from("TODO: should be error details"))
                        .build();
                    if let Err(e) = tx.unbounded_send(sending) {
                        error!("respond REQUEST_RESPONSE failed: {}", e);
                    }
                }
            };
        });
    }

    #[inline]
    async fn on_request_stream(&self, sid: u32, flag: u16, input: Payload) {
        let responder = self.responder.clone();
        let tx = self.tx.clone();
        let splitter = self.splitter.clone();
        self.rt.spawn(async move {
            // TODO: support cancel
            let mut payloads = responder.request_stream(input);
            while let Some(next) = payloads.next().await {
                match next {
                    Ok(it) => {
                        Self::try_send_payload(&splitter, &tx, sid, it, frame::FLAG_NEXT);
                    }
                    Err(e) => {
                        let sending = frame::Error::builder(sid, 0)
                            .set_code(error::ERR_APPLICATION)
                            .set_data(Bytes::from(format!("{}", e)))
                            .build();
                        tx.unbounded_send(sending)
                            .expect("Send stream response failed");
                    }
                };
            }
            let complete = frame::Payload::builder(sid, frame::FLAG_COMPLETE).build();
            tx.unbounded_send(complete)
                .expect("Send stream complete response failed");
        });
    }

    #[inline]
    async fn on_request_channel(&self, sid: u32, flag: u16, first: Payload) {
        let responder = self.responder.clone();
        let tx = self.tx.clone();
        let (sender, receiver) = new_tx_rx::<Result<Payload, RSocketError>>();
        sender.unbounded_send(Ok(first)).unwrap();
        self.register_handler(sid, Handler::ReqRC(sender)).await;
        self.rt.spawn(async move {
            // respond client channel
            let mut outputs = responder.request_channel(Box::pin(receiver));
            // TODO: support custom RequestN.
            let request_n = frame::RequestN::builder(sid, 0).build();

            if let Err(e) = tx.unbounded_send(request_n) {
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
                        .set_code(error::ERR_APPLICATION)
                        .set_data(Bytes::from(format!("{}", e)))
                        .build(),
                };
                tx.unbounded_send(sending).unwrap();
            }
            let complete = frame::Payload::builder(sid, frame::FLAG_COMPLETE).build();
            if let Err(e) = tx.unbounded_send(complete) {
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
        if let Err(e) = tx.unbounded_send(sending.build()) {
            error!("respond KEEPALIVE failed: {}", e);
        }
    }

    #[inline]
    fn try_send_channel(
        splitter: &Option<Splitter>,
        tx: &Tx<Frame>,
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
                            frame::RequestChannel::builder(sid, flag | frame::FLAG_FOLLOW)
                                .set_all(cur.split())
                                .build()
                        } else {
                            frame::Payload::builder(sid, frame::FLAG_FOLLOW)
                                .set_all(cur.split())
                                .build()
                        };
                        // send frame
                        if let Err(e) = tx.unbounded_send(sending) {
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
                if let Err(e) = tx.unbounded_send(sending) {
                    error!("send request_channel failed: {}", e);
                }
            }
            None => {
                let sending = frame::RequestChannel::builder(sid, flag)
                    .set_all(res.split())
                    .build();
                if let Err(e) = tx.unbounded_send(sending) {
                    error!("send request_channel failed: {}", e);
                }
            }
        }
    }

    #[inline]
    fn try_send_payload(
        splitter: &Option<Splitter>,
        tx: &Tx<Frame>,
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
                            frame::Payload::builder(sid, flag | frame::FLAG_FOLLOW)
                                .set_all(cur.split())
                                .build()
                        } else {
                            frame::Payload::builder(sid, frame::FLAG_FOLLOW)
                                .set_all(cur.split())
                                .build()
                        };
                        // send frame
                        if let Err(e) = tx.unbounded_send(sending) {
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
                if let Err(e) = tx.unbounded_send(sending) {
                    error!("send payload failed: {}", e);
                }
            }
            None => {
                let sending = frame::Payload::builder(sid, flag)
                    .set_all(res.split())
                    .build();
                if let Err(e) = tx.unbounded_send(sending) {
                    error!("respond failed: {}", e);
                }
            }
        }
    }
}

impl<R> RSocket for DuplexSocket<R>
where
    R: Send + Sync + Clone + Spawner + 'static,
{
    fn metadata_push(&self, req: Payload) -> Mono<()> {
        let sid = self.seq.next();
        let tx = self.tx.clone();
        Box::pin(async move {
            let (_d, m) = req.split();
            let mut bu = frame::MetadataPush::builder(sid, 0);
            if let Some(b) = m {
                bu = bu.set_metadata(b);
            }
            if let Err(e) = tx.unbounded_send(bu.build()) {
                error!("send metadata_push failed: {}", e);
            }
        })
    }
    fn fire_and_forget(&self, req: Payload) -> Mono<()> {
        let sid = self.seq.next();
        let tx = self.tx.clone();
        let splitter = self.splitter.clone();
        Box::pin(async move {
            match splitter {
                Some(sp) => {
                    let mut cuts: usize = 0;
                    let mut prev: Option<Payload> = None;
                    for next in sp.cut(req, 0) {
                        if let Some(cur) = prev.take() {
                            let sending = if cuts == 1 {
                                // make first frame as request_fnf.
                                frame::RequestFNF::builder(sid, frame::FLAG_FOLLOW)
                                    .set_all(cur.split())
                                    .build()
                            } else {
                                // make other frames as payload.
                                frame::Payload::builder(sid, frame::FLAG_FOLLOW)
                                    .set_all(cur.split())
                                    .build()
                            };
                            // send frame
                            if let Err(e) = tx.unbounded_send(sending) {
                                error!("send fire_and_forget failed: {}", e);
                                return;
                            }
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
                    if let Err(e) = tx.unbounded_send(sending) {
                        error!("send fire_and_forget failed: {}", e);
                    }
                }
                None => {
                    let sending = frame::RequestFNF::builder(sid, 0)
                        .set_all(req.split())
                        .build();
                    if let Err(e) = tx.unbounded_send(sending) {
                        error!("send fire_and_forget failed: {}", e);
                    }
                }
            }
        })
    }

    fn request_response(&self, req: Payload) -> Mono<Result<Payload, RSocketError>> {
        let (tx, rx) = new_tx_rx_once::<Result<Payload, RSocketError>>();
        let sid = self.seq.next();
        let handlers = Arc::clone(&self.get_handler(sid));
        let sender = self.tx.clone();

        let splitter = self.splitter.clone();

        self.rt.spawn(async move {
            {
                // register handler
                let mut map = handlers.lock().await;
                (*map).insert(sid, Handler::ReqRR(tx));
            }

            match splitter {
                Some(sp) => {
                    let mut cuts: usize = 0;
                    let mut prev: Option<Payload> = None;
                    for next in sp.cut(req, 0) {
                        if let Some(cur) = prev.take() {
                            let sending = if cuts == 1 {
                                // make first frame as request_response.
                                frame::RequestResponse::builder(sid, frame::FLAG_FOLLOW)
                                    .set_all(cur.split())
                                    .build()
                            } else {
                                // make other frames as payload.
                                frame::Payload::builder(sid, frame::FLAG_FOLLOW)
                                    .set_all(cur.split())
                                    .build()
                            };
                            // send frame
                            if let Err(e) = sender.unbounded_send(sending) {
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
                    if let Err(e) = sender.unbounded_send(sending) {
                        error!("send request_response failed: {}", e);
                    }
                }
                None => {
                    // crate request frame
                    let sending = frame::RequestResponse::builder(sid, 0)
                        .set_all(req.split())
                        .build();
                    // send frame
                    if let Err(e) = sender.unbounded_send(sending) {
                        error!("send request_response failed: {}", e);
                    }
                }
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
        let handlers = Arc::clone(&self.get_handler(sid));
        let splitter = self.splitter.clone();
        self.rt.spawn(async move {
            {
                let mut map = handlers.lock().await;
                (*map).insert(sid, Handler::ReqRS(sender));
            }
            match splitter {
                Some(sp) => {
                    let mut cuts: usize = 0;
                    let mut prev: Option<Payload> = None;
                    // skip 4 bytes. (initial_request_n is u32)
                    for next in sp.cut(input, 4) {
                        if let Some(cur) = prev.take() {
                            let sending: Frame = if cuts == 1 {
                                // make first frame as request_stream.
                                frame::RequestStream::builder(sid, frame::FLAG_FOLLOW)
                                    .set_all(cur.split())
                                    .build()
                            } else {
                                // make other frames as payload.
                                frame::Payload::builder(sid, frame::FLAG_FOLLOW)
                                    .set_all(cur.split())
                                    .build()
                            };
                            // send frame
                            if let Err(e) = tx.unbounded_send(sending) {
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
                    if let Err(e) = tx.unbounded_send(sending) {
                        error!("send request_stream failed: {}", e);
                    }
                }
                None => {
                    let sending = frame::RequestStream::builder(sid, 0)
                        .set_all(input.split())
                        .build();
                    if let Err(e) = tx.unbounded_send(sending) {
                        error!("send request_stream failed: {}", e);
                    }
                }
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
        let handlers = Arc::clone(&self.get_handler(sid));
        let splitter = self.splitter.clone();
        self.rt.spawn(async move {
            {
                let mut map = handlers.lock().await;
                (*map).insert(sid, Handler::ReqRC(sender));
            }
            let mut first = true;
            while let Some(next) = reqs.next().await {
                match next {
                    Ok(it) => {
                        if first {
                            first = false;
                            Self::try_send_channel(&splitter, &tx, sid, it, frame::FLAG_NEXT)
                        } else {
                            Self::try_send_payload(&splitter, &tx, sid, it, frame::FLAG_NEXT)
                        }
                    }
                    Err(e) => {
                        let sending = frame::Error::builder(sid, 0)
                            .set_code(error::ERR_APPLICATION)
                            .set_data(Bytes::from(format!("{}", e)))
                            .build();
                        if let Err(e) = tx.unbounded_send(sending) {
                            error!("send REQUEST_CHANNEL failed: {}", e);
                        }
                    }
                };
            }
            let sending = frame::Payload::builder(sid, frame::FLAG_COMPLETE).build();
            if let Err(e) = tx.unbounded_send(sending) {
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
