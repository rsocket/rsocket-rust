extern crate futures;

use crate::frame::Frame;
use futures::sync::mpsc::{Receiver, Sender};
use futures::Future;

pub struct Transport {
  _tx: Sender<Frame>,
  _rx: Receiver<Frame>,
}

pub type Context = (Transport, Box<dyn Future<Item = (), Error = ()> + Send>);

impl Transport {
  pub fn new(tx: Sender<Frame>, rx: Receiver<Frame>) -> Transport {
    Transport { _tx: tx, _rx: rx }
  }

  pub fn split(self) -> (Sender<Frame>,Receiver<Frame>){
    (self._tx,self._rx)
  }

}
