extern crate rsocket_rust;

use rsocket_rust::mime::WellKnownMIME;

#[test]
fn test_wellknown() {
    let got = WellKnownMIME::from("application/json");
    assert_eq!(WellKnownMIME::ApplicationJson, got);
    WellKnownMIME::foreach(|m| {
        let mut result = WellKnownMIME::from(m.raw());
        assert_eq!(m, &result);
        result = WellKnownMIME::from(format!("{}", m));
        assert_eq!(m, &result);
    });
}
