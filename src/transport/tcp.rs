use super::codec::RFrameCodec;
use super::spi::{Rx, Tx};
use crate::frame::Frame;
use futures::{Sink, SinkExt, Stream, StreamExt};
use std::net::SocketAddr;
use std::net::TcpStream as StdTcpStream;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc;
use tokio_util::codec::{Decoder, Encoder, Framed, FramedParts, FramedRead, FramedWrite};

pub fn connect(addr: &SocketAddr) -> TcpStream {
  let origin = StdTcpStream::connect(addr).unwrap();
  TcpStream::from_std(origin).unwrap()
}

pub async fn process(socket: TcpStream, mut inputs: Rx, outputs: Tx) {
  let mut stream = Framed::new(socket, RFrameCodec);
  loop {
    match stream.next().await {
      Some(it) => outputs.send(it.unwrap()).unwrap(),
      None => {
        drop(outputs);
        break;
      }
    }
    // TODO: How to split R/W ???
    // match inputs.recv().await {
    //   Some(v) => stream.send(v).await.unwrap(),
    //   None => (),
    // }
  }
}
