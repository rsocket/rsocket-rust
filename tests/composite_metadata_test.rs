extern crate rsocket_rust;

use bytes::{Buf, BufMut, Bytes, BytesMut};
use rsocket_rust::extension::*;
use rsocket_rust::mime::WellKnownMIME;
use rsocket_rust::prelude::*;

#[test]
fn composite_metadata_codec_well() {
    let cm = CompositeMetadata::new(String::from("application/json"), Bytes::from("foobar"));
    composite_metadata_codec(cm);
}

#[test]
fn composite_metadata_codec_bad() {
    let cm = CompositeMetadata::new(String::from("application/now_well"), Bytes::from("foobar"));
    composite_metadata_codec(cm);
}

fn composite_metadata_codec(cm: CompositeMetadata) {
    let mut b = BytesMut::new();
    cm.write_to(&mut b);
    println!("=====> encode bytes: {}", hex::encode(b.to_vec()));
    let cm2 = CompositeMetadata::decode(&mut b).unwrap();
    assert_eq!(1, cm2.len());
    assert_eq!(cm, cm2[0]);
}

#[test]
fn encode_composite_metadata() {
    let cm1 = CompositeMetadata::new(
        String::from("application/json"),
        Bytes::from("hello world!"),
    );
    let mut b = BytesMut::new();
    cm1.write_to(&mut b);
    let cm2 = CompositeMetadata::new(String::from("application/foobar"), Bytes::from("foobar"));
    cm2.write_to(&mut b);
    let results = CompositeMetadata::decode(&mut b).unwrap();
    assert_eq!(2, results.len());
    assert_eq!(cm1, results[0]);
    assert_eq!(cm2, results[1]);
}
