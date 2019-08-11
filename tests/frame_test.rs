extern crate bytes;
extern crate rsocket;

use bytes::Bytes;
use rsocket::frame::*;

#[test]
fn test_build_setup() {
  let setup = Setup::builder(1234, 0)
    .set_data(Bytes::from(String::from("Hello World!")))
    .set_metadata(Bytes::from(String::from("foobar")))
    .build();
  match &setup.body {
    Body::Setup(v) => println!("=======> frame: {:?}", v),
    _ => unimplemented!(),
  };

  let bb = setup.to_bytes();
  println!("bytes: {:?}", bb);
}

