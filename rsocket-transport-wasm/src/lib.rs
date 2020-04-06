#![allow(clippy::type_complexity)]

#[macro_use]
extern crate serde_derive;

mod client;
mod misc;
mod runtime;

pub use client::WebsocketClientTransport;
pub use misc::{connect, new_payload, JsClient, JsPayload};
pub use runtime::WASMSpawner;
