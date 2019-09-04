#![allow(unused_variables)]
#![allow(unused_imports)]
#![allow(dead_code)]

#[macro_use]
extern crate log;
extern crate bytes;
extern crate env_logger;
extern crate futures;
extern crate rsocket_rust;
extern crate tokio;

use bytes::Bytes;
use futures::prelude::*;
use rsocket_rust::prelude::*;

fn main() {
  env_logger::builder()
    .default_format_timestamp_nanos(true)
    .init();

  let server = RSocketFactory::receive()
    .transport(URI::Tcp("127.0.0.1:7878"))
    .acceptor(|setup, sending_socket| {
      info!("accept setup: {:?}", setup);
      // TODO: use tokio runtime?
      // std::thread::spawn(move || {
      //   let resp = sending_socket
      //     .request_response(
      //       Payload::builder()
      //         .set_data(Bytes::from("Hello Client!"))
      //         .build(),
      //     )
      //     .wait()
      //     .unwrap();
      //   println!(">>>>> response success: {:?}", resp);
      // });
      Box::new(MockResponder)
    })
    .serve();
  tokio::run(server);
}
