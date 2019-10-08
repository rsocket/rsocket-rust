extern crate rsocket_rust;

use rsocket_rust::extension::WellKnownMIME;

#[test]
fn test_wellknown() {
    let good = WellKnownMIME::from(0x00);
    assert_eq!(WellKnownMIME::ApplicationAvro, good);
    let bad = WellKnownMIME::from(0x99);
    assert_eq!(WellKnownMIME::Unknown, bad);
}
