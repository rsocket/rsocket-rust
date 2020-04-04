#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(clippy::type_complexity)]
#![doc(test(
    no_crate_inject,
    attr(deny(warnings, rust_2018_idioms), allow(dead_code, unused_variables))
))]

//! Official RSocket Rust implementation using Tokio.
//!
//! RSocket is an application protocol providing Reactive Streams semantics.
//!
//! It is a binary protocol for use on byte stream transports such as TCP, WebSockets, and Aeron.
//!
//! It enables the following symmetric interaction models via async message passing over a single connection:
//! - request/response (stream of 1)
//! - request/stream (finite stream of many)
//! - fire-and-forget (no response)
//! - channel (bi-directional streams)
//!
//! # A Tour of RSocket
//!
//! The easiest way to get started is to use RSocket. Do this by enabling TCP transport support.
//!
//! ```toml
//! rsocket_rust = "0.5"
//! rsocket_rust_transport_tcp = "0.5"
//!
//! # If you want to use websocket transport instead.
//! # rsocket_rust_transport_websocket = "0.5"
//! ```
//!
//! # Examples
//!
//! A simple TCP echo server:
//!
//! ```no_run,ignore
//! use rsocket_rust::prelude::*;
//! use rsocket_rust_transport_tcp::TcpServerTransport;
//! use std::error::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
//!     RSocketFactory::receive()
//!         .transport(TcpServerTransport::from("127.0.0.1:7878"))
//!         .acceptor(Box::new(|setup, socket| {
//!             println!("socket establish: setup={:?}", setup);
//!             tokio::spawn(async move {
//!                 let req = Payload::builder().set_data_utf8("Hello World!").build();
//!                 let res = socket.request_response(req).await.unwrap();
//!                 println!("SERVER request CLIENT success: response={:?}", res);
//!             });
//!             // Return a responder.
//!             // You can write you own responder by implementing `RSocket` trait.
//!             Ok(Box::new(EchoRSocket))
//!         }))
//!         .on_start(Box::new(|| println!("echo server start success!")))
//!         .serve()
//!         .await
//! }
//! ```
//!
//! Connect to echo server above:
//!
//! ```no_run,ignore
//! use rsocket_rust::prelude::*;
//! use rsocket_rust_transport_tcp::TcpClientTransport;
//! use std::error::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
//!     let client = RSocketFactory::connect()
//!         .transport(TcpClientTransport::from("127.0.0.1:7878"))
//!         .acceptor(Box::new(|| {
//!             // Return a responder.
//!             Box::new(EchoRSocket)
//!         }))
//!         .start()
//!         .await
//!         .expect("Connect failed!");
//!
//!     let req = Payload::builder().set_data_utf8("Ping!").build();
//!     let res = client.request_response(req).await.expect("Requet failed!");
//!     println!("request success: response={:?}", res);
//!
//!     Ok(())
//! }
//! ```
//!

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

pub mod error;
pub mod extension;

#[cfg(feature = "frame")]
pub mod frame;
#[cfg(not(feature = "frame"))]
mod frame;

pub mod mime;
mod payload;
pub mod prelude;
pub mod runtime;
mod spi;
pub mod transport;
pub mod utils;
mod x;
