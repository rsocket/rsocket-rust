extern crate rsocket_rust;

use bytes::{Bytes, BytesMut};
use rsocket_rust::extension::RoutingMetadata;

#[test]
fn routing_metadata_codec() {
    let m = RoutingMetadata::builder()
        .push_str("/orders")
        .push_str("/orders/77778888")
        .push_str("/users")
        .push_str("/users/1234")
        .build();

    let mut bf = BytesMut::new();
    m.write_to(&mut bf);
    println!("encode routing metadata: {}", hex::encode(bf.to_vec()));
    let m2 = RoutingMetadata::decode(&mut bf).unwrap();
    let tags = m2.get_tags();
    for tag in tags {
        println!("decode tag: {}", tag);
    }
    assert_eq!(4, tags.len());
    assert_eq!(m.get_tags(), tags);
}
