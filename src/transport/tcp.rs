extern crate futures;
extern crate tokio;

use crate::frame::Frame;
use crate::transport::FrameCodec;
use crate::transport::{Context, Transport};
use futures::sync::mpsc;
use futures::{lazy, Future, Sink, Stream};
use std::io;
use std::net::SocketAddr;
use tokio::codec::Framed;
use tokio::net::TcpStream;

pub fn from_addr(addr: &SocketAddr) -> Context {
  let socket = TcpStream::connect(addr).wait().unwrap();
  from_socket(socket)
}

pub fn from_socket(socket: TcpStream) -> Context {
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
  let fu = lazy(move || {
    tokio::spawn(sender.forward(sink).then(|_| Ok(())));
    task
  });
  (Transport::new(tx, rrx), Box::new(fu))
}
