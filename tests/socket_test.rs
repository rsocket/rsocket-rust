extern crate bytes;
extern crate futures;
extern crate rsocket;
extern crate tokio;

use bytes::Bytes;
use futures::future::ok;
use futures::stream::iter_ok;
use futures::{Future, Stream};
use rsocket::prelude::*;

struct MockResponder;

impl RSocket for MockResponder {
  fn metadata_push(&self, req: Payload) -> Box<Future<Item = (), Error = RSocketError>> {
    println!("receive metadata_push: {:?}", req);
    Box::new(ok(()).map_err(|()| RSocketError::from("foobar")))
  }

  fn request_fnf(&self, req: Payload) -> Box<Future<Item = (), Error = RSocketError>> {
    println!("receive request_fnf: {:?}", req);
    Box::new(ok(()).map_err(|()| RSocketError::from("foobar")))
  }

  fn request_response(&self, req: Payload) -> Box<Future<Item = Payload, Error = RSocketError>> {
    println!(">>>>>>>> responder: {:?}", req);
    Box::new(ok(req))
  }

  fn request_stream(&self, req: Payload) -> Box<Stream<Item = Payload, Error = RSocketError>> {
    println!(">>>>>>>> accept stream: {:?}", req);
    let mut results = vec![];
    for n in 0..10 {
      let pa = Payload::builder()
        .set_data(Bytes::from(format!("DATA_{}", n)))
        .set_metadata(Bytes::from(format!("METADATA_{}", n)))
        .build();
      results.push(pa);
    }
    Box::new(iter_ok(results))
  }
}

#[test]
fn test_socket_request() {
  // Prepare a Echo server. (see: https://github.com/rsocket/rsocket-go/tree/master/cmd/echo)

  // create a socket.
  let socket = DuplexSocket::builder("127.0.0.1:7878")
    .set_setup(Payload::from("Ready!"))
    .set_acceptor(Box::new(MockResponder))
    .connect();

  // metadata push
  socket
    .metadata_push(
      Payload::builder()
        .set_metadata(Bytes::from("metadata only!"))
        .build(),
    )
    .wait()
    .unwrap();

  // request fnf
  let fnf = Payload::from("Mock FNF");
  socket.request_fnf(fnf).wait().unwrap();

  // request response
  for n in 0..3 {
    let sending = Payload::builder()
      .set_data(Bytes::from(format!("[{}] Hello Rust!", n)))
      .set_metadata(Bytes::from("text/plain"))
      .build();
    let result = socket.request_response(sending).wait().unwrap();
    println!("********** YES: {:?}", result);
  }

  // request stream
  let sending = Payload::builder()
    .set_data(Bytes::from("Hello Rust!"))
    .set_metadata(Bytes::from("text/plain"))
    .build();
  let task = socket
    .request_stream(sending)
    .map_err(|_| ())
    .for_each(|it| {
      println!("******* STREAM: {:?}", it);
      Ok(())
    });
  task.wait().unwrap();
}
