extern crate bytes;
extern crate futures;
extern crate rsocket_rust;

use bytes::Bytes;
use futures::prelude::*;
use rsocket_rust::prelude::*;

#[test]
fn test_serve() {
  RSocketFactory::receive()
    .transport(URI::Tcp("127.0.0.1:7878"))
    .acceptor(|setup, sending_socket| {
      println!("accept setup: {:?}", setup);
      // TODO: use tokio runtime?
      std::thread::spawn(move || {
        let resp = sending_socket
          .request_response(
            Payload::builder()
              .set_data(Bytes::from("Hello Client!"))
              .build(),
          )
          .wait()
          .unwrap();
        println!(">>>>> response success: {:?}", resp);
      });
      Box::new(MockResponder)
    })
    .serve()
    .wait()
    .unwrap();
}
