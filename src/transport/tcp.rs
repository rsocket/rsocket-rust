extern crate futures;
extern crate tokio;

use super::{Context, FrameCodec, Transport};
use crate::frame::Frame;
use futures::sync::mpsc;
use futures::{lazy, Future, Sink, Stream};
use std::io;
use std::net::SocketAddr;
use std::net::TcpStream as StdTcpStream;
use tokio::codec::Framed;
use tokio::net::TcpStream;
use tokio::reactor::Handle;

pub fn from_addr(addr: &SocketAddr) -> Context {
  let origin = StdTcpStream::connect(addr).unwrap();
  let socket = TcpStream::from_std(origin, &Handle::default()).unwrap();
  // let socket = TcpStream::connect(addr).wait().unwrap();
  from_socket(socket)
}

pub fn from_socket(socket: TcpStream) -> Context {
  let (tx_snd, rx_snd) = mpsc::channel::<Frame>(0);
  let (tx_rcv, rx_rcv) = mpsc::channel::<Frame>(0);
  let (sink, stream) = Framed::new(socket, FrameCodec::new()).split();
  let sender: Box<dyn Stream<Item = Frame, Error = io::Error> + Send> =
    Box::new(rx_snd.map_err(|_| panic!("errors not possible on rx")));
  let tx_rcv_gen = move || tx_rcv.clone();
  let task = stream
    .for_each(move |it| {
      tx_rcv_gen().send(it).wait().unwrap();
      Ok(())
    })
    .map_err(|e| println!("error reading: {:?}", e));
  let fu = lazy(move || {
    tokio::spawn(sender.forward(sink).then(|_| Ok(())));
    task
  });
  (Transport::new(tx_snd, rx_rcv), Box::new(fu))
}
