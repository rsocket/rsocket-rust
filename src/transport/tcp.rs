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
  tx: Sender<Frame>,
  rx: Receiver<Frame>,
}

impl Context {

  pub fn get_tx(&self) -> Sender<Frame> {
    self.tx.clone()
  }

  pub fn get_rx(self) -> Receiver<Frame> {
    self.rx
  }
}

impl From<&'static str> for Context{

  fn from(addr: &'static str) -> Context{
    let socket_addr:SocketAddr = addr.parse().unwrap();
    Context::from(&socket_addr)
  }

}

impl From<&SocketAddr> for Context{

  fn from(addr: &SocketAddr) -> Context{
    let (tx, rx) = mpsc::channel(0);
    let (rtx, rrx) = mpsc::channel(0);
    let task = TcpStream::connect(addr)
      .and_then(|socket| {
        let (sink, stream) = Framed::new(socket, FrameCodec::new()).split();
        let sender: Box<dyn Stream<Item = Frame, Error = io::Error> + Send> =
          Box::new(rx.map_err(|_| panic!("errors not possible on rx")));
        tokio::spawn(sender.forward(sink).then(|result| Ok(())));
        stream.for_each(move |it| {
          rtx.clone().send(it).wait().unwrap();
          Ok(())
        })
      })
      .map_err(|e| println!("error reading: {:?}", e));
    // TODO: how do run future daemon???
    std::thread::spawn(move || {
      tokio::run(task);
    });
    Context { tx: tx, rx: rrx }
  }
}
