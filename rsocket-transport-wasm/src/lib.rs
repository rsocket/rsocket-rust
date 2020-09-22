#![allow(clippy::type_complexity)]

#[macro_use]
extern crate serde_derive;

mod client;
mod connection;
mod misc;

pub use client::WebsocketClientTransport;
pub use connection::WebsocketConnection;
pub use misc::{connect, new_payload, JsClient, JsPayload};
