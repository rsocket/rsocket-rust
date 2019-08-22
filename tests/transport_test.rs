extern crate bytes;
extern crate futures;
extern crate rsocket;
extern crate tokio;

use bytes::Bytes;
use futures::sync::{mpsc, oneshot};
use rsocket::frame;
use rsocket::prelude::*;
use rsocket::transport::*;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::thread;
use tokio::prelude::*;

#[test]
fn mock_conn() {
  let c = Conn::new();
  c.register(frame::TYPE_CANCEL, |it| {
    println!("accept cancel: {:?}", it);
  });

  let f = frame::Cancel::new(1234, 0);
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

#[derive(Debug)]
struct Stores {
  map: Mutex<HashMap<u32, oneshot::Sender<Payload>>>,
}

#[test]
fn test_dial() {
  // NOTICE:
  // prepare test echo server, you can install golang rsocket-cli tool and run command below.
  // rsocket-cli --request -i world -s --debug tcp://127.0.0.1:7878

  let ctx = Context::builder("127.0.0.1:7878").build();
  let tx = ctx.get_tx();
  let (sender, caller) = RequestCaller::new();
  thread::spawn(move || {
    next_send(tx);
  });

  let mut store: HashMap<u32, oneshot::Sender<Payload>> = HashMap::new();
  store.insert(1 as u32, sender);
  let holder = Arc::new(Stores {
    map: Mutex::new(store),
  });

  tokio::run(futures::lazy(|| {
    tokio::spawn(
      caller
        .and_then(|x: Payload| {
          // BINGO!!!!
          println!("@@@@@@@@@@@@@@@@@@@@@@@@@@@@@: {:?}", x);
          Ok(())
        })
        .map_err(|_| ()),
    );

    // consume all incoming frames
    ctx.get_rx().for_each(move |f: frame::Frame| {
      println!("incoming: {:?}", f);
      let holder = holder.clone();
      match f.get_body() {
        frame::Body::Payload(v) => {
          println!("incoming payload frame body: {:?}", v);
          let mut bu = Payload::builder();
          match v.get_data() {
            Some(vv) => {
              bu.set_data(vv);
              ()
            }
            None => (),
          };

          match v.get_metadata() {
            Some(vv) => {
              bu.set_metadata(vv);
              ()
            }
            None => (),
          };
          let pa = bu.build();
          println!("*************** bingo payload: {:?}", pa);
          // pick handler
          let mut senders = holder.map.lock().unwrap();
          let got = senders.remove(&f.get_stream_id()).unwrap();
          println!("*************** got handler: {:?}", got);
          // fire event!
          got.send(pa).unwrap();
        }
        _ => unimplemented!(),
      };
      Ok(())
    })
  }));
}

fn next_send(mut tx: mpsc::Sender<frame::Frame>) {
  let setup = frame::Setup::builder(0, 0)
    .set_data(Bytes::from("Hello Rust"))
    .build();
  tx = tx.send(setup).wait().unwrap();
  let req = frame::RequestResponse::builder(1, 0)
    .set_data(Bytes::from("xxxx"))
    .set_metadata(Bytes::from("yyyy"))
    .build();
  tx = tx.send(req).wait().unwrap();
  loop {
    thread::sleep(std::time::Duration::from_secs(3));
    tx = match tx
      .send(frame::Keepalive::builder(0, frame::FLAG_RESPOND).build())
      .wait()
    {
      Ok(tx) => tx,
      Err(_) => break,
    }
  }
}
