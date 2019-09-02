extern crate futures;
extern crate rsocket_rust;
extern crate tokio;

use futures::{Future, Stream};
use rsocket_rust::prelude::*;

#[test]
fn test_socket_request() {
  // Prepare a Echo server. (see: https://github.com/rsocket/rsocket-go/tree/master/cmd/echo)
  let addr = "127.0.0.1:7878".parse().unwrap();
  // create a socket.
  let socket = DuplexSocket::builder()
    .set_acceptor(Acceptor::Direct(Box::new(MockResponder)))
    .connect(&addr);

  // setup manual
  let setup = SetupPayload::builder().set_data_utf8("Ready!").build();
  socket.setup(setup).wait().unwrap();

  // metadata push
  socket
    .metadata_push(
      Payload::builder()
        .set_metadata_utf8("metadata only!")
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
      .set_data_utf8("Hello Rust!")
      .set_metadata_utf8(&format!("#{}", n))
      .build();
    let result = socket.request_response(sending).wait().unwrap();
    println!("******* REQUEST: {:?}", result);
  }

  // request stream
  let sending = Payload::builder()
    .set_data_utf8("Hello Rust!")
    .set_metadata_utf8("foobar")
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
