extern crate rsocket_rust;

use rsocket_rust::extension::{self, MimeType};

#[test]
fn test_wellknown() {
    let well = MimeType::from("application/json");
    assert_eq!(extension::MimeType::APPLICATION_JSON, well);
    assert_eq!(0x05, well.as_u8().unwrap());
    let custom = MimeType::from("application/custom");
    assert_eq!(MimeType::Normal("application/custom".to_owned()), custom);
}
