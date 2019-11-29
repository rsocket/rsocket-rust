extern crate bytes;
extern crate hex;
extern crate rsocket_rust;

use bytes::{Bytes, BytesMut};
use rsocket_rust::frame::*;

#[test]
fn test_setup() {
  let f = Setup::builder(1234, 0)
    .set_mime_data("application/binary")
    .set_mime_metadata("text/plain")
    .set_token(Bytes::from("this_is_a_token"))
    .set_data(Bytes::from(String::from("Hello World!")))
    .set_metadata(Bytes::from(String::from("foobar")))
    .build();
  try_codec(f);
}

#[test]
fn test_keepalive() {
  let f = Keepalive::builder(1234, FLAG_RESPOND)
    .set_last_received_position(123)
    .set_data(Bytes::from("foobar"))
    .build();
  try_codec(f);
}

#[test]
fn test_request_response() {
  let f = RequestResponse::builder(1234, 0)
    .set_data(Bytes::from("Hello World"))
    .set_metadata(Bytes::from("Foobar"))
    .build();
  try_codec(f);
}

#[test]
fn test_payload() {
  let f = Payload::builder(1234, FLAG_NEXT | FLAG_COMPLETE)
    .set_data(Bytes::from("Hello World!"))
    .set_metadata(Bytes::from("foobar"))
    .build();
  try_codec(f);
}

#[test]
fn test_request_channel() {
  let f = RequestChannel::builder(1234, 0)
    .set_initial_request_n(1)
    .set_data(Bytes::from("Hello World!"))
    .set_metadata(Bytes::from("foobar"))
    .build();
  try_codec(f);
}

#[test]
fn test_cancel() {
  let f = Cancel::builder(1234, 0).build();
  try_codec(f);
}

#[test]
fn test_request_fnf() {
  let f = RequestFNF::builder(1234, 0)
    .set_data(Bytes::from("Hello"))
    .set_metadata(Bytes::from("World"))
    .build();
  try_codec(f);
}

#[test]
fn test_metadata_push() {
  let f = MetadataPush::builder(1234, 0)
    .set_metadata(Bytes::from("Hello Rust!"))
    .build();
  try_codec(f);
}

#[test]
fn test_request_n() {
  let f = RequestN::builder(1234, 0).set_n(77778888).build();
  try_codec(f);
}

#[test]
fn test_lease() {
  let f = Lease::builder(1234, 0)
    .set_metadata(Bytes::from("Hello Rust!"))
    .set_number_of_requests(333)
    .set_ttl(1000)
    .build();
  try_codec(f);
}

#[test]
fn test_error() {
  let f = Error::builder(1234, 0)
    .set_data(Bytes::from("Hello World!"))
    .set_code(4444)
    .build();
  try_codec(f);
}

#[test]
fn resume_ok() {
  let f = ResumeOK::builder(1234, 0).set_position(2333).build();
  try_codec(f);
}

#[test]
fn test_resume() {
  let f = Resume::builder(0, FLAG_RESUME)
    .set_last_received_server_position(123)
    .set_first_available_client_position(22)
    .set_token(Bytes::from("this is a token"))
    .build();
  try_codec(f);
}

fn try_codec(f: Frame) {
  println!("******* codec: {:?}", f);
  let mut bf = BytesMut::with_capacity(f.len() as usize);
  f.write_to(&mut bf);
  println!("####### encode: {}", hex::encode(bf.to_vec()));
  let f2 = Frame::decode(&mut bf).unwrap();
  println!("####### decode: {:?}", f2);
  assert_eq!(
    f, f2,
    "frames doesn't match: expect={:?}, actual={:?}",
    f, f2
  );
}
