#![allow(clippy::upper_case_acronyms)]

mod misc;
mod requester;

pub use misc::{cbor, json, SerDe};
pub use requester::{RequestSpec, Requester, RequesterBuilder};
