extern crate futures;
extern crate tokio;

use crate::frame::Frame;
use crate::transport::FrameCodec;
// use futures::sync::mpsc::{channel, Receiver, Sender};
use futures::sync::mpsc;
use futures::Sink;
use std::io::{self, Read, Write};
use std::net::SocketAddr;
use tokio::codec::Framed;
use tokio::net::TcpStream;
use tokio::prelude::*;

pub fn dial(addr: &SocketAddr, ch: Box<dyn Stream<Item = Frame, Error = io::Error> + Send>) {
  // let (tx,rx) = mpsc::channel::<Frame>(10);
  // let rx = rx.map_err(|_| panic!("errors not possible on rx"));
  let task = TcpStream::connect(addr)
    .and_then(|socket| {
      let (sink, stream) = Framed::new(socket, FrameCodec::new()).split();
      tokio::spawn(ch.forward(sink).then(|result| Ok(())));
      stream.for_each(|f| {
        println!("received frame {:?}", f);
        Ok(())
      })
    })
    .map_err(|err| println!("error: {:?}", err));
  tokio::run(task)
}
