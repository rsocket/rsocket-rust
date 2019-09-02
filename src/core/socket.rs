extern crate bytes;
extern crate futures;
extern crate tokio;

use crate::core::misc::StreamID;
use crate::core::spi::RSocket;
use crate::core::{RequestCaller, StreamCaller};
use crate::errors::{ErrorKind, RSocketError};
use crate::frame::{self, Body, Frame};
use crate::payload::{Payload, SetupPayload};
use crate::transport::{self, Context};

use bytes::Bytes;
use futures::sync::{mpsc, oneshot};
use futures::{future, lazy, stream};
use futures::{Future, Sink, Stream};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex, RwLock};
use tokio::net::TcpStream;

pub enum Acceptor {
  Direct(Box<dyn RSocket>),
  Generate(Arc<fn(SetupPayload, Box<dyn RSocket>) -> Box<dyn RSocket>>),
  Empty(),
}

#[derive(Debug)]
enum Handler {
  Request(oneshot::Sender<Payload>),
  Stream(mpsc::Sender<Payload>),
}

#[derive(Debug)]
struct Handlers {
  map: Mutex<HashMap<u32, Handler>>,
}

impl Handlers {
  fn new() -> Handlers {
    Handlers {
      map: Mutex::new(HashMap::new()),
    }
  }
}

#[derive(Clone)]
pub struct DuplexSocket {
  tx: mpsc::Sender<Frame>,
  seq: StreamID,
  handlers: Arc<Handlers>,
}

// #[derive(Clone)]
struct Runner {
  tx: mpsc::Sender<Frame>,
  handlers: Arc<Handlers>,
  responder: Responder,
  acceptor: Acceptor,
  socket: DuplexSocket,
}

impl Runner {
  fn new(
    tx: mpsc::Sender<Frame>,
    handlers: Arc<Handlers>,
    acceptor: Acceptor,
    socket: DuplexSocket,
  ) -> Runner {
    let (responder, acceptor) = match acceptor {
      Acceptor::Direct(v) => (Responder::from(v), Acceptor::Empty()),
      Acceptor::Empty() => (Responder::new(), Acceptor::Empty()),
      Acceptor::Generate(x) => (Responder::new(), Acceptor::Generate(x)),
    };
    Runner {
      tx: tx,
      handlers: handlers,
      acceptor: acceptor,
      responder: responder,
      socket: socket,
    }
  }

  fn respond_metadata_push(&self, input: Payload) {
    let responder = self.responder.clone();
    tokio::spawn(lazy(move || {
      responder.metadata_push(input).wait().unwrap();
      Ok(())
    }));
  }

  fn respond_fnf(&self, input: Payload) {
    let responder = self.responder.clone();
    tokio::spawn(lazy(move || {
      responder.request_fnf(input).wait().unwrap();
      Ok(())
    }));
  }

  fn respond_request_response(&self, sid: u32, flag: u16, input: Payload) {
    let responder = self.responder.clone();
    let tx = self.tx.clone();

    tokio::spawn(lazy(move || {
      let result = responder
        .request_response(input)
        .map(|res| {
          let mut bu = frame::Payload::builder(sid, frame::FLAG_COMPLETE);
          if let Some(b) = res.data() {
            bu = bu.set_data(b);
          }
          if let Some(b) = res.metadata() {
            bu = bu.set_metadata(b);
          }
          bu.build()
        })
        .wait();
      let sending = match result {
        Ok(sending) => sending,
        Err(e) => frame::Error::builder(sid, 0)
          .set_code(frame::ERR_APPLICATION)
          .set_data(Bytes::from("TODO: should be error details"))
          .build(),
      };
      tx.send(sending)
        .map(|_| ())
        .map_err(|e| println!("error: {}", e))
    }));
  }

  fn respond_request_stream(&self, sid: u32, flag: u16, input: Payload) {
    let responder = self.responder.clone();
    let tx = self.tx.clone();
    tokio::spawn(lazy(move || {
      let tx2 = tx.clone();
      responder
        .request_stream(input)
        .map(|elem| {
          let mut bu = frame::Payload::builder(sid, frame::FLAG_NEXT);
          if let Some(b) = elem.data() {
            bu = bu.set_data(b);
          }
          if let Some(b) = elem.metadata() {
            bu = bu.set_metadata(b);
          }
          bu.build()
        })
        .forward(tx)
        .and_then(|_| {
          let complete = frame::Payload::builder(sid, frame::FLAG_COMPLETE).build();
          tx2.send(complete).map_err(|_e| unreachable!())
        })
        .wait()
        .unwrap();
      Ok(())
    }));
  }

  fn to_future(self, rx: mpsc::Receiver<Frame>) -> impl Future<Item = (), Error = ()> + Send {
    let handlers = self.handlers.clone();
    let task = rx.for_each(move |f: frame::Frame| {
      // println!("[DEBUG] incoming: {:?}", f);
      let sid = f.get_stream_id();
      let body = f.get_body();
      match body {
        Body::Setup(v) => {
          let pa = SetupPayload::from(v);
          match &self.acceptor {
            Acceptor::Generate(f) => {
              let rs = Box::new(self.socket.clone());
              let r = f(pa, rs);
              self.responder.set(r);
            }
            _ => unimplemented!(),
          };
        }
        Body::Payload(v) => {
          let pa = Payload::from(v);
          // pick handler
          let mut senders = handlers.map.lock().unwrap();
          let handler = senders.remove(&sid).unwrap();

          let mut tx1: Option<oneshot::Sender<Payload>> = None;
          let mut tx2: Option<mpsc::Sender<Payload>> = None;

          // fire event!
          match handler {
            Handler::Request(sender) => {
              tx1 = Some(sender);
              ()
            }
            Handler::Stream(sender) => {
              if f.has_next() {
                tx2 = Some(sender.clone());
              }
              if !f.has_complete() {
                senders.insert(f.get_stream_id(), Handler::Stream(sender));
              }
              ()
            }
          };

          if let Some(sender) = tx1 {
            sender.send(pa).unwrap();
          } else if let Some(sender) = tx2 {
            sender.send(pa).wait().unwrap();
          }
        }
        Body::RequestResponse(v) => {
          let pa = Payload::from(v);
          let flag = f.get_flag();
          self.respond_request_response(sid, flag, pa);
        }
        Body::RequestStream(v) => {
          let pa = Payload::from(v);
          let flag = f.get_flag();
          self.respond_request_stream(sid, flag, pa);
        }
        _ => {
          println!("incoming unsupported frame: {:?}", f);
        }
      };
      Ok(())
    });

    task
    // std::thread::spawn(move || {
    //   tokio::run(task);
    // });
  }
}

impl DuplexSocket {
  pub fn builder() -> DuplexSocketBuilder {
    DuplexSocketBuilder::new()
  }

  fn new(first_stream_id: u32, ctx: Context, responder: Acceptor) -> DuplexSocket {
    let tp = ctx.0;
    let task0 = ctx.1;

    let handlers = Arc::new(Handlers::new());
    let handlers2 = handlers.clone();
    let tx1 = tp.tx();
    let tx2 = tp.tx();

    let sk = DuplexSocket {
      tx: tx1,
      handlers: handlers,
      seq: StreamID::from(first_stream_id),
    };

    let task = Runner::new(tx2, handlers2, responder, sk.clone()).to_future(tp.rx());

    // TODO: Daemon???
    std::thread::spawn(move || {
      tokio::run(lazy(move || {
        tokio::spawn(task0);
        task
      }));
    });

    sk
  }

  pub fn setup(&self, setup: SetupPayload) -> impl Future<Item = (), Error = RSocketError> {
    let mut bu = frame::Setup::builder(0, 0);

    if let Some(b) = setup.data() {
      bu = bu.set_data(b);
    }
    if let Some(b) = setup.metadata() {
      bu = bu.set_metadata(b);
    }
    if let Some(s) = setup.data_mime_type() {
      bu = bu.set_mime_data(&s);
    }
    if let Some(s) = setup.metadata_mime_type() {
      bu = bu.set_mime_metadata(&s);
    }
    let sending = bu
      .set_keepalive(setup.keepalive_interval())
      .set_lifetime(setup.keepalive_lifetime())
      .build();
    self.send_frame(sending)
  }

  fn send_frame(&self, sending: Frame) -> Box<dyn Future<Item = (), Error = RSocketError>> {
    let tx = self.tx.clone();
    let task = tx
      .send(sending)
      .map(|_| ())
      .map_err(|e| RSocketError::from("send frame failed"));
    Box::new(task)
  }

  fn register_handler(&self, sid: u32, handler: Handler) {
    let handlers: Arc<Handlers> = self.handlers.clone();
    let mut senders = handlers.map.lock().unwrap();
    senders.insert(sid, handler);
  }
}

impl RSocket for DuplexSocket {
  fn metadata_push(&self, req: Payload) -> Box<dyn Future<Item = (), Error = RSocketError>> {
    let sid = self.seq.next();
    let mut bu = frame::MetadataPush::builder(sid, 0);
    if let Some(b) = req.metadata() {
      bu = bu.set_metadata(b);
    }
    let sending = bu.build();
    let tx = self.tx.clone();
    let fu = tx
      .send(sending)
      .map(|_| ())
      .map_err(|e| RSocketError::from(e));
    Box::new(fu)
  }

  fn request_fnf(&self, req: Payload) -> Box<dyn Future<Item = (), Error = RSocketError>> {
    let sid = self.seq.next();
    let mut bu = frame::RequestFNF::builder(sid, 0);
    if let Some(b) = req.data() {
      bu = bu.set_data(b);
    }
    if let Some(b) = req.metadata() {
      bu = bu.set_metadata(b);
    }
    let sending = bu.build();
    let tx = self.tx.clone();
    let fu = tx
      .send(sending)
      .map(|_| ())
      .map_err(|e| RSocketError::from(e));
    Box::new(fu)
  }

  fn request_response(
    &self,
    input: Payload,
  ) -> Box<dyn Future<Item = Payload, Error = RSocketError>> {
    let sid = self.seq.next();
    let (tx, caller) = RequestCaller::new();
    // register handler
    self.register_handler(sid, Handler::Request(tx));
    // crate request frame
    let mut bu = frame::RequestResponse::builder(sid, 0);
    if let Some(b) = input.data() {
      bu = bu.set_data(b);
    }
    if let Some(b) = input.metadata() {
      bu = bu.set_metadata(b);
    }
    let sending = bu.build();
    println!("......ready: {:?}", sending);
    // send frame
    self.tx.clone().send(sending).wait().unwrap();
    // tokio::spawn(emitter.send(sent).and_then(|_| Ok(())).map_err(|_| ()));
    Box::new(caller)
  }

  fn request_stream(
    &self,
    input: Payload,
  ) -> Box<dyn Stream<Item = Payload, Error = RSocketError>> {
    let sid = self.seq.next();
    // register handler
    let (tx, caller) = StreamCaller::new();
    self.register_handler(sid, Handler::Stream(tx));
    // crate stream frame
    let mut bu = frame::RequestStream::builder(sid, 0);
    if let Some(b) = input.data() {
      bu = bu.set_data(b);
    }
    if let Some(b) = input.metadata() {
      bu = bu.set_metadata(b);
    }
    let sending = bu.build();
    self.tx.clone().send(sending).wait().unwrap();
    Box::new(caller)
  }
}

pub struct DuplexSocketBuilder {
  acceptor: Acceptor,
}

impl DuplexSocketBuilder {
  fn new() -> DuplexSocketBuilder {
    DuplexSocketBuilder {
      acceptor: Acceptor::Empty(),
    }
  }

  pub fn set_acceptor(mut self, acceptor: Acceptor) -> Self {
    self.acceptor = acceptor;
    self
  }

  pub fn from_socket(self, socket: TcpStream) -> DuplexSocket {
    let ctx = transport::from_socket(socket);
    self.build(ctx, 2)
  }

  pub fn connect(self, addr: &SocketAddr) -> DuplexSocket {
    let ctx = transport::from_addr(addr);
    self.build(ctx, 1)
  }

  fn build(self, ctx: Context, starter: u32) -> DuplexSocket {
    DuplexSocket::new(starter, ctx, self.acceptor)
  }
}

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
  fn metadata_push(&self, req: Payload) -> Box<dyn Future<Item = (), Error = RSocketError>> {
    let inner = self.inner.clone();
    let r = inner.read().unwrap();
    (*r).metadata_push(req)
  }

  fn request_fnf(&self, req: Payload) -> Box<dyn Future<Item = (), Error = RSocketError>> {
    let inner = self.inner.clone();
    let r = inner.read().unwrap();
    (*r).request_fnf(req)
  }

  fn request_response(
    &self,
    req: Payload,
  ) -> Box<dyn Future<Item = Payload, Error = RSocketError>> {
    let inner = self.inner.clone();
    let r = inner.read().unwrap();
    (*r).request_response(req)
  }

  fn request_stream(&self, req: Payload) -> Box<dyn Stream<Item = Payload, Error = RSocketError>> {
    let inner = self.inner.clone();
    let r = inner.read().unwrap();
    (*r).request_stream(req)
  }
}

pub struct EmptyRSocket;

impl EmptyRSocket {
  fn must_failed(&self) -> RSocketError {
    RSocketError::from(ErrorKind::Internal(frame::ERR_APPLICATION, "NOT_IMPLEMENT"))
  }
}

impl RSocket for EmptyRSocket {
  fn metadata_push(&self, _req: Payload) -> Box<dyn Future<Item = (), Error = RSocketError>> {
    Box::new(future::err(self.must_failed()))
  }

  fn request_fnf(&self, _req: Payload) -> Box<dyn Future<Item = (), Error = RSocketError>> {
    Box::new(future::err(self.must_failed()))
  }

  fn request_response(
    &self,
    _req: Payload,
  ) -> Box<dyn Future<Item = Payload, Error = RSocketError>> {
    Box::new(future::err(self.must_failed()))
  }

  fn request_stream(&self, _req: Payload) -> Box<dyn Stream<Item = Payload, Error = RSocketError>> {
    Box::new(stream::iter_result(Err(self.must_failed())))
  }
}
