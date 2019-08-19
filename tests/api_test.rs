extern crate bytes;
extern crate rsocket;

use bytes::Bytes;
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
