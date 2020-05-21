extern crate rsocket_rust;

use rsocket_rust::extension::{self, MimeType};

#[test]
fn test_from_str() {
    let result = MimeType::from("foobar");
    match result {
        MimeType::WellKnown(_) => panic!("failed"),
        MimeType::Normal(s) => assert_eq!(&s, "foobar"),
    }
}

#[test]
fn test_wellknown() {
    let well = MimeType::from("application/json");
    assert_eq!(extension::MimeType::APPLICATION_JSON, well);
    assert_eq!(0x05, well.as_u8().unwrap());
    let custom = MimeType::from("application/custom");
    assert_eq!(MimeType::Normal("application/custom".to_owned()), custom);
}

#[test]
fn test_parse() {
    assert!(MimeType::parse(0xFF).is_none(), "should be none");
    for x in 0..0x29 {
        assert!(MimeType::parse(x).is_some(), "should not be none");
    }
    for x in 0x29..0x7A {
        assert!(MimeType::parse(x).is_none(), "should be none");
    }
    for x in 0x7A..0x80 {
        assert!(MimeType::parse(x).is_some(), "should not be none");
    }
    for x in 0x80..0xFF {
        assert!(MimeType::parse(x).is_none(), "should be none");
    }
}
