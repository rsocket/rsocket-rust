use bytes::BytesMut;
use rsocket_rust::extension::{self, CompositeMetadata, CompositeMetadataEntry, MimeType};
use rsocket_rust::utils::Writeable;

#[test]
fn encode_and_decode_composite_metadata() {
    let bingo = |metadatas: Vec<&CompositeMetadataEntry>| {
        assert_eq!(2, metadatas.len());
        assert_eq!(
            extension::MimeType::TEXT_PLAIN,
            *metadatas[0].get_mime_type()
        );
        assert_eq!("Hello World!", metadatas[0].get_metadata_utf8());
        assert_eq!(
            MimeType::from("application/not_well"),
            *metadatas[1].get_mime_type()
        );
        assert_eq!(b"Not Well!", metadatas[1].get_metadata().as_ref());
    };

    let cm = CompositeMetadata::builder()
        .push(MimeType::from("text/plain"), b"Hello World!")
        .push(MimeType::from("application/not_well"), "Not Well!")
        .build();
    bingo(cm.iter().collect());

    let mut bf = BytesMut::new();
    cm.write_to(&mut bf);
    let cm2 = CompositeMetadata::decode(&mut bf).unwrap();
    bingo(cm2.iter().collect());
}
