extern crate bytes;
extern crate futures;

use crate::errors::RSocketError;
use crate::payload::Payload;
use futures::sync::{mpsc, oneshot};
use futures::{Future, Poll, Stream};

pub struct RequestCaller {
  rx: oneshot::Receiver<Payload>,
}

impl RequestCaller {
  pub fn new() -> (oneshot::Sender<Payload>, RequestCaller) {
    let (tx, rx) = oneshot::channel();
    (tx, RequestCaller { rx: rx })
  }
}

impl Future for RequestCaller {
  type Item = Payload;
  type Error = RSocketError;

  fn poll(&mut self) -> Poll<Self::Item, Self::Error> {
    self.rx.poll().map_err(|e| RSocketError::from(e))
  }
}

pub struct StreamCaller {
  rx: mpsc::Receiver<Payload>,
}

impl StreamCaller {
  pub fn new() -> (mpsc::Sender<Payload>, StreamCaller) {
    let (tx, rx) = mpsc::channel(0);
    (tx, StreamCaller { rx: rx })
  }
}

impl Stream for StreamCaller {
  type Item = Payload;
  type Error = RSocketError;

  fn poll(&mut self) -> Poll<Option<Self::Item>, Self::Error> {
    self
      .rx
      .poll()
      .map_err(|()| RSocketError::from("todo: wrap stream poll error"))
  }
}
