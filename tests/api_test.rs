extern crate bytes;
extern crate futures;
extern crate rsocket;
extern crate tokio;

use bytes::Bytes;
use futures::future::lazy;
use futures::sync::mpsc::Sender;
use futures::Future;
use rsocket::prelude::*;

struct MockRSocket;

impl RSocket for MockRSocket {
  fn request_response(&self, msg: Payload) -> Result<Payload> {
    Ok(msg)
  }
}

#[test]
fn test_responder() {
  let p = Payload::builder()
    .set_data(Bytes::from("Hello World!"))
    .set_metadata(Bytes::from("Greet"))
    .build();
  println!("origin: {:?}", p);
  let rsp = Responder::new(MockRSocket);
  match rsp.request_response(p) {
    Ok(v) => println!("invoke: {:?}", v),
    Err(_) => (),
  }
}

#[test]
fn test_payload() {}

#[test]
fn error() {
  let err = ErrorKind::WithDescription("foobar error");
  println!("err: {:?}", err);
}

#[test]
fn test_request_caller() {
  let (tx, caller) = RequestCaller::new();
  std::thread::spawn(move || {
    let pa = Payload::builder()
      .set_data(Bytes::from("Hello Future"))
      .set_metadata(Bytes::from("text/plain"))
      .build();
    tx.send(pa).map_err(|e| ())
  });

  tokio::run(lazy(|| {
    caller
      .and_then(|x| {
        println!("==============> RESULT: {:?}", x);
        Ok(())
      })
      .map_err(|e| ())
  }));
}
