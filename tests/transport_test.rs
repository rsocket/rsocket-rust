extern crate bytes;
extern crate futures;
extern crate rsocket;
extern crate tokio;

use futures::sync::mpsc;

use bytes::Bytes;
use rsocket::frame::*;
use rsocket::transport::*;
use std::sync::Arc;
use std::thread;
use tokio::codec::Framed;
use tokio::net::TcpListener;
use tokio::prelude::*;

#[test]
fn name() {
  let c = Conn::new();
  c.register(TYPE_CANCEL, |it| {
    println!("accept cancel: {:?}", it);
  });

  let f = Cancel::new(1234, 0);
  c.bingo(&f);
}

// #[test]
// fn test_serve() {
//   // serve and echo each incoming frame.
//   let addr = "127.0.0.1:7878".parse().unwrap();
//   let listener = TcpListener::bind(&addr).unwrap();
//   let server = listener
//     .incoming()
//     .for_each(|socket| {
//       let framed_sock = Framed::new(socket, FrameCodec::new());
//       framed_sock.for_each(|f| {
//         println!("received frame {:?}", f);
//         Ok(())
//       })
//     })
//     .map_err(|err| println!("error: {:?}", err));
//   tokio::run(server);
// }

#[test]
fn test_dial() {
  let (tx, rx) = mpsc::channel(0);
  thread::spawn(|| next_send(tx));

  let rx = rx.map_err(|_| panic!("errors not possible on rx"));
  let addr = "127.0.0.1:7878".parse().unwrap();
  dial(&addr, Box::new(rx));
}

fn next_send(mut tx: mpsc::Sender<Frame>) {
  let setup = Setup::builder(1, 0)
    .set_data(Bytes::from("Hello Rust"))
    .build();
  tx = match tx.send(setup).wait() {
    Ok(tx) => tx,
    Err(_) => return,
  };
  loop {
    thread::sleep(std::time::Duration::from_secs(3));
    tx = match tx.send(Keepalive::builder(0, FLAG_RESPOND).build()).wait() {
      Ok(tx) => tx,
      Err(_) => break,
    }
  }
}
