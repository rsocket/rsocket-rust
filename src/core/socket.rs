extern crate bytes;
extern crate futures;
extern crate tokio;

use crate::core::misc::StreamID;
use crate::core::spi::RSocket;
use crate::core::{RequestCaller, StreamCaller};
use crate::errors::{ErrorKind, RSocketError};
use crate::frame::{self, Body, Frame};
use crate::mime::MIME_BINARY;
use crate::payload::{Payload, SetupPayload};
use crate::transport::Context;

use bytes::Bytes;
use futures::sync::{mpsc, oneshot};
use futures::{future, lazy, stream};
use futures::{Future, Sink, Stream};
use std::collections::HashMap;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tokio::net::TcpStream;
// use tokio::runtime::Runtime;

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

pub struct DuplexSocket {
  tx: mpsc::Sender<Frame>,
  seq: StreamID,
  handlers: Arc<Handlers>,
}

pub struct DuplexSocketBuilder {
  acceptor: Arc<Box<dyn RSocket>>,
}

#[derive(Clone)]
struct Runner {
  tx: mpsc::Sender<Frame>,
  handlers: Arc<Handlers>,
  responder: Arc<Box<dyn RSocket>>,
}

impl Runner {
  fn new(
    tx: mpsc::Sender<Frame>,
    handlers: Arc<Handlers>,
    responder: Arc<Box<dyn RSocket>>,
  ) -> Runner {
    Runner {
      tx,
      handlers,
      responder,
    }
  }

  fn respond_metadata_push(&self, input: Payload) {
    let responder = self.responder.clone();
    // TODO: use future spawn
    std::thread::spawn(move || {
      responder.metadata_push(input).wait().unwrap();
    });
  }

  fn respond_fnf(&self, input: Payload) {
    let responder = self.responder.clone();
    // TODO: use future spawn
    std::thread::spawn(move || {
      responder.request_fnf(input).wait().unwrap();
    });
  }

  fn respond_request_response(&self, sid: u32, flag: u16, input: Payload) {
    let responder = self.responder.clone();
    let tx = self.tx.clone();
    // TODO: use future spawn

    tokio::spawn(lazy(move || {
      let result = responder
        .request_response(input)
        .map(|res| {
          let mut bu = frame::Payload::builder(sid, frame::FLAG_COMPLETE);
          if let Some(b) = res.data() {
            bu.set_data(b);
          }
          if let Some(b) = res.metadata() {
            bu.set_metadata(b);
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
      // tx.send(sending).wait().unwrap();
      // // let sending =
      // //
      // Ok(())
    }));

    // let res = responder.request_response(input).wait().unwrap();

    // println!(">>> respond: {:?}", sending);
    // self.tx.clone().send(sending).wait().unwrap();
    // .and_then(|sending|{
    //     let bu = frame::Payload::builder(sid, frame::FLAG_COMPLETE);
    //       tx.send(bu.build())
    // });
  }

  fn respond_request_stream(&self, sid: u32, flag: u16, input: Payload) {
    let responder = self.responder.clone();
    let tx = self.tx.clone();
    // TODO: use future spawn
    std::thread::spawn(move || {
      let tx2 = tx.clone();
      let stream = responder
        .request_stream(input)
        .map(|elem| {
          let mut bu = frame::Payload::builder(sid, frame::FLAG_NEXT);
          if let Some(b) = elem.data() {
            bu.set_data(b);
          }
          if let Some(b) = elem.metadata() {
            bu.set_metadata(b);
          }
          bu.build()
        })
        .map_err(|e| {
          println!("respond request stream failed: {}", e);
          unimplemented!()
        });
      tx.send_all(stream).wait().unwrap();
      let complete = frame::Payload::builder(sid, frame::FLAG_COMPLETE).build();
      tx2.clone().send(complete).wait().unwrap();
    });
  }

  fn run(self, rx: mpsc::Receiver<Frame>) {
    let handlers = self.handlers.clone();
    let task = rx.for_each(move |f: frame::Frame| {
      // println!("[DEBUG] incoming: {:?}", f);
      let sid = f.get_stream_id();
      let body = f.get_body();
      match body {
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
    std::thread::spawn(move || {
      tokio::run(task);
    });
  }
}

impl DuplexSocket {
  pub fn builder() -> DuplexSocketBuilder {
    DuplexSocketBuilder::new()
  }

  fn new(first_stream_id: u32, ctx: Context, responder: Arc<Box<dyn RSocket>>) -> DuplexSocket {
    let handlers = Arc::new(Handlers::new());
    let handlers2 = handlers.clone();
    let tx1 = ctx.tx();
    let tx2 = ctx.tx();
    let runner = Runner::new(tx2, handlers2, responder);
    runner.run(ctx.rx());
    DuplexSocket {
      tx: tx1,
      handlers: handlers,
      seq: StreamID::from(first_stream_id),
    }
  }

  pub fn setup(&self, setup: SetupPayload) -> Box<Future<Item = (), Error = RSocketError>> {
    let mut bu = frame::Setup::builder(0, 0);

    if let Some(b) = setup.data() {
      bu.set_data(b);
    }
    if let Some(b) = setup.metadata() {
      bu.set_metadata(b);
    }
    if let Some(s) = setup.data_mime_type() {
      bu.set_mime_data(&s);
    }
    if let Some(s) = setup.metadata_mime_type() {
      bu.set_mime_metadata(&s);
    }
    let sending = bu
      .set_keepalive(setup.keepalive_interval())
      .set_lifetime(setup.keepalive_lifetime())
      .build();
    self.send_frame(sending)
  }

  fn send_frame(&self, sending: Frame) -> Box<Future<Item = (), Error = RSocketError>> {
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
  fn metadata_push(&self, req: Payload) -> Box<Future<Item = (), Error = RSocketError>> {
    let sid = self.seq.next();
    let mut bu = frame::MetadataPush::builder(sid, 0);
    if let Some(b) = req.metadata() {
      bu.set_metadata(b);
    }
    let sending = bu.build();
    let tx = self.tx.clone();
    Box::new(
      tx.send(sending)
        .map(|_| ())
        .map_err(|e| RSocketError::from("send metadata_push failed")),
    )
  }

  fn request_fnf(&self, req: Payload) -> Box<Future<Item = (), Error = RSocketError>> {
    let sid = self.seq.next();
    let mut bu = frame::RequestFNF::builder(sid, 0);
    if let Some(b) = req.data() {
      bu.set_data(b);
    }
    if let Some(b) = req.metadata() {
      bu.set_metadata(b);
    }
    let sending = bu.build();
    let tx = self.tx.clone();
    Box::new(
      tx.send(sending)
        .map(|_| ())
        .map_err(|e| RSocketError::from("send request FNF failed")),
    )
  }

  fn request_response(&self, input: Payload) -> Box<Future<Item = Payload, Error = RSocketError>> {
    let sid = self.seq.next();
    let (tx, caller) = RequestCaller::new();
    // register handler
    self.register_handler(sid, Handler::Request(tx));
    // crate request frame
    let mut bu = frame::RequestResponse::builder(sid, 0);
    if let Some(b) = input.data() {
      bu.set_data(b);
    }
    if let Some(b) = input.metadata() {
      bu.set_metadata(b);
    }
    let sending = bu.build();
    // send frame
    self.tx.clone().send(sending).wait().unwrap();
    // tokio::spawn(emitter.send(sent).and_then(|_| Ok(())).map_err(|_| ()));
    Box::new(caller)
  }

  fn request_stream(&self, input: Payload) -> Box<Stream<Item = Payload, Error = RSocketError>> {
    let sid = self.seq.next();
    // register handler
    let (tx, caller) = StreamCaller::new();
    self.register_handler(sid, Handler::Stream(tx));
    // crate stream frame
    let mut bu = frame::RequestStream::builder(sid, 0);
    if let Some(b) = input.data() {
      bu.set_data(b);
    }
    if let Some(b) = input.metadata() {
      bu.set_metadata(b);
    }
    let sending = bu.build();
    self.tx.clone().send(sending).wait().unwrap();
    Box::new(caller)
  }
}

impl DuplexSocketBuilder {
  fn new() -> DuplexSocketBuilder {
    DuplexSocketBuilder {
      acceptor: Arc::new(Box::new(EmptyRSocket)),
    }
  }

  pub fn set_acceptor(&mut self, acceptor: Box<dyn RSocket>) -> &mut DuplexSocketBuilder {
    self.acceptor = Arc::new(acceptor);
    self
  }

  pub fn from_socket(&mut self, socket: TcpStream) -> DuplexSocket {
    let ctx = Context::from(socket);
    self.build(ctx, 2)
  }

  pub fn connect(&mut self, addr: &SocketAddr) -> DuplexSocket {
    let ctx = Context::from(addr);
    self.build(ctx, 1)
  }

  fn build(&mut self, ctx: Context, starter: u32) -> DuplexSocket {
    let responder = self.acceptor.clone();
    let sk = DuplexSocket::new(starter, ctx, responder);
    sk
  }
}

struct EmptyRSocket;

impl EmptyRSocket {
  fn must_failed(&self) -> RSocketError {
    RSocketError::from(ErrorKind::Internal(frame::ERR_APPLICATION, "NOT_IMPLEMENT"))
  }
}

impl RSocket for EmptyRSocket {
  fn metadata_push(&self, _req: Payload) -> Box<Future<Item = (), Error = RSocketError>> {
    Box::new(future::err(self.must_failed()))
  }

  fn request_fnf(&self, _req: Payload) -> Box<Future<Item = (), Error = RSocketError>> {
    Box::new(future::err(self.must_failed()))
  }

  fn request_response(&self, _req: Payload) -> Box<Future<Item = Payload, Error = RSocketError>> {
    Box::new(future::err(self.must_failed()))
  }

  fn request_stream(&self, _req: Payload) -> Box<Stream<Item = Payload, Error = RSocketError>> {
    Box::new(stream::iter_result(Err(self.must_failed())))
  }
}
