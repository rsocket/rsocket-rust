extern crate bytes;
extern crate futures;
extern crate tokio;

use crate::core::api::RSocket;
use crate::core::misc::StreamID;
use crate::core::{RequestCaller, StreamCaller};
use crate::errors::RSocketError;
use crate::frame::{self, Body, Frame};
use crate::mime::MIME_BINARY;
use crate::payload::Payload;
use crate::transport::Context;

use bytes::Bytes;
use futures::sync::mpsc;
use futures::sync::oneshot;
use futures::{Future, Sink, Stream};
use std::collections::HashMap;
use std::net::SocketAddr;
use std::sync::{Arc, Mutex};
use std::time::Duration;
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
  addr: String,
  acceptor: Arc<Box<dyn RSocket>>,
  setup: Option<Payload>,
  keepalive_interval: Duration,
  keepalive_lifetime: Duration,
  mime_data: Option<String>,
  mime_metadata: Option<String>,
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

  fn respond_request_response(&self, sid: u32, flag: u16, input: Payload) {
    // TODO: do async
    let responder = self.responder.clone();
    let tx = self.tx.clone();

    std::thread::spawn(move || {
      let sending = responder
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
        .wait()
        .unwrap();
      tx.send(sending).wait().unwrap();
    });
    // let res = responder.request_response(input).wait().unwrap();

    // println!(">>> respond: {:?}", sending);
    // self.tx.clone().send(sending).wait().unwrap();
    // .and_then(|sending|{
    //     let bu = frame::Payload::builder(sid, frame::FLAG_COMPLETE);
    //       tx.send(bu.build())
    // });
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
  pub fn builder(addr: &str) -> DuplexSocketBuilder {
    DuplexSocketBuilder::new(String::from(addr))
  }

  fn new(ctx: Context, responder: Arc<Box<dyn RSocket>>) -> DuplexSocket {
    let handlers = Arc::new(Handlers::new());
    let handlers2 = handlers.clone();
    let tx1 = ctx.get_tx();
    let tx2 = ctx.get_tx();
    let runner = Runner::new(tx2, handlers2, responder);
    runner.run(ctx.get_rx());
    DuplexSocket {
      tx: tx1,
      handlers: handlers,
      seq: StreamID::from(1),
    }
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
  fn new(addr: String) -> DuplexSocketBuilder {
    DuplexSocketBuilder {
      addr: addr,
      acceptor: Arc::new(Box::new(EmptyRSocket)),
      setup: None,
      keepalive_interval: Duration::from_secs(20),
      keepalive_lifetime: Duration::from_secs(90),
      mime_data: Some(String::from(MIME_BINARY)),
      mime_metadata: Some(String::from(MIME_BINARY)),
    }
  }

  pub fn set_acceptor(&mut self, acceptor: Box<dyn RSocket>) -> &mut DuplexSocketBuilder {
    self.acceptor = Arc::new(acceptor);
    self
  }

  pub fn set_setup(&mut self, setup: Payload) -> &mut DuplexSocketBuilder {
    self.setup = Some(setup);
    self
  }

  pub fn set_keepalive(
    &mut self,
    tick_period: Duration,
    ack_timeout: Duration,
    missed_acks: u64,
  ) -> &mut DuplexSocketBuilder {
    let lifetime_mills = (ack_timeout.as_millis() as u64) * missed_acks;
    self.keepalive_interval = tick_period;
    self.keepalive_lifetime = Duration::from_millis(lifetime_mills);
    self
  }

  pub fn set_data_mime_type(&mut self, mime: String) -> &mut DuplexSocketBuilder {
    self.mime_data = Some(mime);
    self
  }
  pub fn set_metadata_mime_type(&mut self, mime: String) -> &mut DuplexSocketBuilder {
    self.mime_metadata = Some(mime);
    self
  }

  pub fn connect(&mut self) -> DuplexSocket {
    let addr: SocketAddr = self.addr.parse().unwrap();
    let ctx = Context::from(&addr);
    let responder = self.acceptor.clone();
    let sk = DuplexSocket::new(ctx, responder);
    let mut bu = frame::Setup::builder(0, 0);
    match &self.setup {
      Some(v) => {
        if let Some(b) = v.data() {
          bu.set_data(b);
        }
        if let Some(b) = v.metadata() {
          bu.set_metadata(b);
        }
        ()
      }
      None => (),
    };
    if let Some(s) = &self.mime_data {
      bu.set_mime_data(&s);
    }
    if let Some(s) = &self.mime_metadata {
      bu.set_mime_metadata(&s);
    }
    let sending = bu
      .set_keepalive(self.keepalive_interval)
      .set_lifetime(self.keepalive_lifetime)
      .build();
    sk.send_frame(sending).wait().unwrap();
    sk
  }
}

struct EmptyRSocket;

impl RSocket for EmptyRSocket {
  fn metadata_push(&self, req: Payload) -> Box<Future<Item = (), Error = RSocketError>> {
    unimplemented!()
  }

  fn request_fnf(&self, req: Payload) -> Box<Future<Item = (), Error = RSocketError>> {
    unimplemented!()
  }

  fn request_response(&self, req: Payload) -> Box<Future<Item = Payload, Error = RSocketError>> {
    unimplemented!()
  }

  fn request_stream(&self, req: Payload) -> Box<Stream<Item = Payload, Error = RSocketError>> {
    unimplemented!()
  }
}
