extern crate bytes;
extern crate hex;
extern crate rsocket;

use bytes::{BigEndian, BufMut, Bytes, BytesMut};
use rsocket::frame::*;

#[test]
fn test_build_setup() {
  let f = Setup::builder(1234, 0)
    .set_data(Bytes::from(String::from("Hello World!")))
    .set_metadata(Bytes::from(String::from("foobar")))
    .build();
  match f.get_body() {
    Body::Setup(v) => println!("=======> setup_frame: {:?}", v),
    _ => unimplemented!(),
  };
  let mut bf = BytesMut::with_capacity(2048);
  f.write_to(&mut bf);
  let bs = bf.take().freeze().to_vec();
  println!("======> hex: {}", hex::encode(bs));
  // println!("bytes: {:?}", );
}