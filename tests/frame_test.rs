extern crate bytes;
extern crate hex;
extern crate rsocket;

use bytes::{BigEndian, BufMut, Bytes, BytesMut};
use rsocket::frame::*;

#[test]
fn test_build_setup() {
  let f = Setup::builder(1234, 0)
    .set_mime_data(String::from("application/binary"))
    .set_mime_metadata(String::from("text/plain"))
    .set_data(Bytes::from(String::from("Hello World!")))
    .set_metadata(Bytes::from(String::from("foobar")))
    .build();
  println!("=======> build: {:?}", f);
  println!("***** frame size: {}", f.len());
  let mut bf = BytesMut::with_capacity(f.len() as usize);
  f.write_to(&mut bf);
  let mut bb = bf.take().freeze();
  println!("======> hex: {}", hex::encode(bb.to_vec()));
  let f2 = Frame::decode(&mut bb).unwrap();
  println!("----> decode: {:?}", f2)
}
