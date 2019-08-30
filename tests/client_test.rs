extern crate futures;
extern crate rsocket_rust;

use futures::prelude::*;
use rsocket_rust::prelude::*;

#[test]
fn test_client() {
  let cli = RSocketFactory::connect()
    .acceptor(Box::new(MockResponder))
    .transport(URI::Tcp("127.0.0.1:7878"))
    .setup(Payload::from("READY!"))
    .mime_type("text/plain", "text/plain")
    .start()
    .unwrap();
  let pa = Payload::builder()
    .set_data_utf8("Hello World!")
    .set_metadata_utf8("Rust!")
    .build();
  let resp = cli.request_response(pa).wait().unwrap();
  println!("******* response: {:?}", resp);
  // exec(cli);
}

fn exec(socket: impl RSocket) {
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
