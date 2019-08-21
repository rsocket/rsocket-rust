extern crate futures;
extern crate tokio;

use crate::frame::Frame;
use crate::result::Result;
use crate::transport::FrameCodec;
use futures::future::Future;
use futures::sync::mpsc::{self, Receiver, Sender};
use futures::{Sink, Stream};
use std::cell::RefCell;
use std::error::Error;
use std::io;
use std::net::SocketAddr;
use tokio::codec::Framed;
use tokio::net::TcpStream;
use tokio::prelude::*;

pub struct Context {
  tx: Sender<Frame>,
  rx: Receiver<Frame>,
}

impl Context {
  pub fn builder(addr: &'static str) -> ContextBuilder {
    ContextBuilder::new(addr)
  }

  pub fn get_tx(&self) -> Sender<Frame> {
    self.tx.clone()
  }

  pub fn get_rx(self) -> Receiver<Frame> {
    self.rx
  }
}

pub struct ContextBuilder {
  addr: &'static str,
  handler: Option<Box<Fn(&Context, Frame)>>,
}

impl ContextBuilder {
  fn new(addr: &'static str) -> ContextBuilder {
    ContextBuilder {
      addr: addr,
      handler: None,
    }
  }

  pub fn build(&mut self) -> Context {
    let addr2 = self.addr.parse().unwrap();
    let (tx, rx) = mpsc::channel(0);
    let (rtx, rrx) = mpsc::channel(0);

    let task = TcpStream::connect(&addr2)
      .and_then(|socket| {
        println!("oooo");
        match Self::process(rx, socket) {
          Ok(stream) => {
            println!("dsafasdf");
            stream.for_each(move |it| {
              rtx.clone().send(it).wait().unwrap();
              Ok(())
            })
          }
          Err(e) => unimplemented!(),
        }
      })
      .map_err(|e| println!("error reading: {:?}", e));
    // TODO: how do run future daemon???
    std::thread::spawn(move || {
      tokio::run(task);
    });
    Context { tx: tx, rx: rrx }
  }

  fn process(
    rx: Receiver<Frame>,
    socket: TcpStream,
  ) -> Result<impl Stream<Item = Frame, Error = io::Error>> {
    let (sink, stream) = Framed::new(socket, FrameCodec::new()).split();
    let sender: Box<dyn Stream<Item = Frame, Error = io::Error> + Send> =
      Box::new(rx.map_err(|_| panic!("errors not possible on rx")));
    tokio::spawn(sender.forward(sink).then(|result| {
      println!("hahaha");
      Ok(())
    }));
    Ok(stream)
  }
}

// pub fn connect(addr: &SocketAddr) -> (Sender<Frame>, impl Future<Item = (), Error = ()>) {
//   let (tx,rx) = mpsc::unbounded();

//   let task = TcpStream::connect(addr)
//     .and_then(|socket| match process(socket) {
//       Ok((tx, stream)) => stream.for_each(|f| {
//         println!("incoming frame: {:?}", f);
//         Ok(())
//       }),
//       Err(e) => unimplemented!(),
//     })
//     .map_err(|e| println!("error reading: {:?}", e));
//   (tx,task)
// }


