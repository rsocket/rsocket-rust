extern crate rsocket_rust;
extern crate futures;

use futures::{Future};
use rsocket_rust::prelude::*;


#[test]
fn test_client() {
  let cli = Client::builder().set_uri("127.0.0.1:7878").build().unwrap();
  let resp = cli.request_response(Payload::builder().set_data_utf8("Hello World!").set_metadata_utf8("Rust!").build()).wait().unwrap();
  println!("response: {:?}",resp);
}
