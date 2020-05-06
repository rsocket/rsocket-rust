#[macro_use]
extern crate log;

mod misc;
mod requester;

pub use requester::{RequestSpec, Requester, RequesterBuilder};
