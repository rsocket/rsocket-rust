#![allow(clippy::type_complexity)]

#[macro_use]
extern crate log;

mod client;
mod server;

pub use client::WebsocketClientTransport;
pub use server::WebsocketServerTransport;
