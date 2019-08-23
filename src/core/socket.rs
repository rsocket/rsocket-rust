extern crate bytes;
extern crate futures;
extern crate tokio;

use crate::core::misc::StreamID;
use crate::core::{RequestCaller, StreamCaller};
use crate::errors::RSocketError;
use crate::frame::{self, Body, Frame};
use crate::payload::Payload;
use crate::transport::Context;

use bytes::Bytes;
use futures::sync::mpsc;
use futures::sync::oneshot;
use futures::{Future, Sink, Stream};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
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
  handlers: Arc<Handlers>,
  seq: StreamID,
}

impl DuplexSocket {
  pub fn connect(addr: &'static str) -> DuplexSocket {
    let ctx = Context::builder(addr).build();
    Self::new(ctx)
  }

  fn new(ctx: Context) -> DuplexSocket {
    let handlers = Arc::new(Handlers::new());
    let tx = ctx.get_tx();
    let handlers_cloned = handlers.clone();

    // TODO: how to schedule future as daemon???
    Self::run(handlers_cloned, ctx);

    DuplexSocket {
      tx: tx,
      handlers: handlers,
      seq: StreamID::from(1),
    }
  }

  fn run(handlers: Arc<Handlers>, ctx: Context) {
    let task = ctx.get_rx().for_each(move |f: frame::Frame| {
      // println!("[DEBUG] incoming: {:?}", f);
      let body = f.get_body();
      match body {
        Body::Payload(v) => {
          let pa = Payload::from(v);
          // pick handler
          let mut senders = handlers.map.lock().unwrap();
          let handler = senders.remove(&f.get_stream_id()).unwrap();

          let mut tx1: Option<oneshot::Sender<Payload>> = None;
          let mut tx2: Option<mpsc::Sender<Payload>> = None;

          // fire event!
          // got.send(pa).unwrap();
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

  pub fn setup(&self, setup: Payload) -> impl Future<Item = (), Error = RSocketError> {
    let mut bu = frame::Setup::builder(0, 0);
    match setup.data() {
      Some(b) => {
        bu.set_data(b);
        ()
      }
      None => (),
    };
    match setup.metadata() {
      Some(b) => {
        bu.set_metadata(b);
        ()
      }
      None => (),
    };
    let sending = bu.build();
    let tx = self.tx.clone();
    tx.send(sending)
      .map(|_| ())
      .map_err(|e| RSocketError::from("send setup frame failed"))
  }

  pub fn request_response(
    &self,
    input: Payload,
  ) -> impl Future<Item = Payload, Error = RSocketError> {
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
    caller
  }

  pub fn request_stream(
    &self,
    input: Payload,
  ) -> impl Stream<Item = Payload, Error = RSocketError> {
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
    caller
  }

  fn register_handler(&self, sid: u32, handler: Handler) {
    let handlers: Arc<Handlers> = self.handlers.clone();
    let mut senders = handlers.map.lock().unwrap();
    senders.insert(sid, handler);
  }
}

impl From<Context> for DuplexSocket {
  fn from(ctx: Context) -> DuplexSocket {
    DuplexSocket::new(ctx)
  }
}
