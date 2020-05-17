#[macro_use]
extern crate log;
// #[macro_use]
// extern crate lazy_static;

mod misc;
mod requester;

pub use misc::{cbor, json, SerDe};
pub use requester::{RequestSpec, Requester, RequesterBuilder};
