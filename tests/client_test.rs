extern crate futures;
extern crate rsocket_rust;

use futures::prelude::*;
use rsocket_rust::prelude::*;

#[test]
fn test_client() {
  let cli = Client::builder()
    .set_uri("127.0.0.1:7878")
    .set_setup(Payload::from("READY!"))
    .set_data_mime_type("text/plain")
    .set_metadata_mime_type("text/plain")
    .build()
    .unwrap();
  let pa = Payload::builder()
    .set_data_utf8("Hello World!")
    .set_metadata_utf8("Rust!")
    .build();
  let resp = cli.request_response(pa).wait().unwrap();
  println!("====> response: {:?}", resp);
}
