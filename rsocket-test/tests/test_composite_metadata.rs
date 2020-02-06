use bytes::{Buf, BufMut, Bytes, BytesMut};
use rsocket_rust::extension::{CompositeMetadata, Metadata};
use rsocket_rust::utils::Writeable;

#[test]
fn encode_and_decode_composite_metadata() {
    let bingo = |metadatas: Vec<&Metadata>| {
        assert_eq!(2, metadatas.len());
        assert_eq!("text/plain", metadatas[0].get_mime());
        assert_eq!(b"Hello World!", metadatas[0].get_payload().as_ref());
        assert_eq!("application/not_well", metadatas[1].get_mime());
        assert_eq!(b"Not Well!", metadatas[1].get_payload().as_ref());
    };

    let cm = CompositeMetadata::builder()
        .push("text/plain", b"Hello World!")
        .push("application/not_well", "Not Well!")
        .build();
    bingo(cm.iter().collect());

    let mut bf = BytesMut::new();
    cm.write_to(&mut bf);
    let cm2 = CompositeMetadata::decode(&mut bf).unwrap();
    bingo(cm2.iter().collect());
}
