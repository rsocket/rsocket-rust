extern crate bytes;
extern crate futures;

use crate::errors::RSocketError;
use crate::frame::{Frame, RequestResponse};
use crate::payload::Payload;
use crate::transport::Context;
use futures::sync::oneshot::{self, Canceled, Receiver, Sender};
use futures::{Async, Future, Poll};

pub struct RequestCaller {
  rx: Receiver<Payload>,
}

impl RequestCaller {
  pub fn new() -> (Sender<Payload>,RequestCaller) {
    let (tx, rx) = oneshot::channel();
    (tx,RequestCaller { rx: rx })
  }
}

impl Future for RequestCaller {
  type Item = Payload;
  type Error = RSocketError;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    self.rx.poll().map_err(|e| RSocketError::from(e))
  }
}
