extern crate bytes;
extern crate hex;
extern crate rsocket;

use bytes::{Bytes, BytesMut};
use rsocket::frame::*;

#[test]
fn test_setup() {
  let f = Setup::builder(1234, 0)
    .set_mime_data("application/binary")
    .set_mime_metadata("text/plain")
    .set_token(Bytes::from("this_is_a_token"))
    .set_data(Bytes::from(String::from("Hello World!")))
    .set_metadata(Bytes::from(String::from("foobar")))
    .build();
  try_encode_decode(&f);
}

#[test]
fn test_keepalive() {
  let ka = Keepalive::builder(1234, FLAG_RESPOND)
    .set_last_received_position(123)
    .set_data(Bytes::from("foobar"))
    .build();
  try_encode_decode(&ka);
}

#[test]
fn test_request_response() {
  let f = RequestResponse::builder(1234, 0)
    .set_data(Bytes::from("Hello World"))
    .set_metadata(Bytes::from("Foobar"))
    .build();
  try_encode_decode(&f);
}

#[test]
fn test_payload() {
  let f = Payload::builder(1234, FLAG_NEXT | FLAG_COMPLETE)
    .set_data(Bytes::from("Hello World!"))
    .set_metadata(Bytes::from("foobar"))
    .build();
  try_encode_decode(&f);
}

fn try_encode_decode(f: &Frame) {
  println!("******* codec: {:?}", f);
  let mut bf = BytesMut::with_capacity(f.len() as usize);
  f.write_to(&mut bf);
  let mut bb = bf.take();
  println!("####### encode: {}", hex::encode(bb.to_vec()));
  let f2 = Frame::decode(&mut bb).unwrap();
  println!("####### decode: {:?}", f2);
}
