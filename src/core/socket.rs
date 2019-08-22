extern crate bytes;
extern crate futures;
extern crate tokio;

use crate::core::RequestCaller;
use crate::errors::RSocketError;
use crate::frame;
use crate::payload::Payload;
use crate::transport::Context;

use bytes::Bytes;

use futures::sync::mpsc;
use futures::sync::oneshot;
use futures::{Future, Sink, Stream};
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use tokio::runtime::Runtime;

pub type Response = Future<Item = Payload, Error = RSocketError>;

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
  tx: mpsc::Sender<frame::Frame>,
  handlers: Arc<Handlers>,
}

impl DuplexSocket {

  pub fn connect(addr: &'static str) -> DuplexSocket {
    let ctx = Context::builder(addr).build();
    Self::new(ctx)
  }

  pub fn new(ctx: Context) -> DuplexSocket {
    let handlers = Arc::new(Handlers::new());
    let tx = ctx.get_tx();
    let handlers_cloned = handlers.clone();

    // TODO: how to schedule future as daemon???
    Self::run(handlers_cloned, ctx);

    DuplexSocket {
      tx: tx,
      handlers: handlers,
    }
  }

  fn run(handlers: Arc<Handlers>, ctx: Context) {
    let task = ctx.get_rx().for_each(move |f: frame::Frame| {
      println!("[DEBUG] incoming: {:?}", f);
      match f.get_body() {
        frame::Body::Payload(v) => {
          let mut bu = Payload::builder();
          match v.get_data() {
            Some(vv) => {
              bu.set_data(vv);
              ()
            }
            None => (),
          };

          match v.get_metadata() {
            Some(vv) => {
              bu.set_metadata(vv);
              ()
            }
            None => (),
          };
          let pa = bu.build();
          // pick handler
          let mut senders = handlers.map.lock().unwrap();
          let got = senders.remove(&f.get_stream_id()).unwrap();
          // fire event!
          // got.send(pa).unwrap();
          match got {
            Handler::Request(sender) => {
              sender.send(pa).unwrap();
              ()
            }
            _ => (),
          };
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

  pub fn request_response(self, input: Payload) -> RequestCaller {
    let (tx, caller) = RequestCaller::new();
    // TODO: stream id generator.
    let mut bu = frame::RequestResponse::builder(1, 0);
    match input.data() {
      Some(bs) => {
        bu.set_data(bs);
        ()
      }
      None => (),
    };
    match input.metadata() {
      Some(bs) => {
        bu.set_metadata(bs);
        ()
      }
      None => (),
    };
    let handlers: Arc<Handlers> = self.handlers.clone();
    let mut senders = handlers.map.lock().unwrap();
    senders.insert(1 as u32, Handler::Request(tx));
    let sent = bu.build();
    // TODO: ensure setup
    let setup = frame::Setup::builder(0, 0)
      .set_data(Bytes::from("Hello Rust"))
      .build();
    self.tx.clone().send(setup).wait().unwrap();
    self.tx.clone().send(sent).wait().unwrap();
    // tokio::spawn(emitter.send(sent).and_then(|_| Ok(())).map_err(|_| ()));
    caller
  }
}
