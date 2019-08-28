extern crate futures;
extern crate tokio;

use crate::frame::Frame;
use crate::transport::FrameCodec;
use futures::future::Future;
use futures::sync::mpsc::{self, Receiver, Sender};
use futures::{Sink, Stream};
use std::io;
use std::net::SocketAddr;
use tokio::codec::Framed;
use tokio::net::TcpStream;

pub struct Context {
  _tx: Sender<Frame>,
  _rx: Receiver<Frame>,
}

impl Context {
  pub fn new(tx: Sender<Frame>, rx: Receiver<Frame>) -> Context {
    Context { _tx: tx, _rx: rx }
  }

  pub fn tx(&self) -> Sender<Frame> {
    self._tx.clone()
  }

  pub fn rx(self) -> Receiver<Frame> {
    self._rx
  }
}

impl From<&'static str> for Context {
  fn from(addr: &'static str) -> Context {
    let socket_addr: SocketAddr = addr.parse().unwrap();
    Context::from(&socket_addr)
  }
}

impl From<TcpStream> for Context {
  fn from(socket: TcpStream) -> Context {
    let (tx, rx) = mpsc::channel(0);
    let (rtx, rrx) = mpsc::channel(0);
    let (sink, stream) = Framed::new(socket, FrameCodec::new()).split();
    let sender: Box<dyn Stream<Item = Frame, Error = io::Error> + Send> =
      Box::new(rx.map_err(|_| panic!("errors not possible on rx")));

    let task = stream
      .for_each(move |it| {
        rtx.clone().send(it).wait().unwrap();
        Ok(())
      })
      .map_err(|e| println!("error reading: {:?}", e));
    // let task = TcpStream::connect(addr)
    //   .and_then(|socket| {

    //   })
    //   .map_err(|e| println!("error reading: {:?}", e));
    // TODO: how do run future daemon???
    std::thread::spawn(move || {
      tokio::run(futures::lazy(|| {
        tokio::spawn(sender.forward(sink).then(|_| Ok(())));
        task
      }));
      // tokio::run(task);
    });
    Context::new(tx, rrx)
  }
}

impl From<&SocketAddr> for Context {
  fn from(addr: &SocketAddr) -> Context {
    let socket = TcpStream::connect(addr).wait().unwrap();
    Context::from(socket)
  }
}
