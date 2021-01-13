#![allow(clippy::type_complexity)]

#[macro_use]
extern crate log;

mod client;
mod connection;
mod misc;
mod server;

#[cfg(feature = "tls")]
pub use client::TlsClientTransport;
pub use client::{TcpClientTransport, UnixClientTransport};
#[cfg(feature = "tls")]
pub use connection::TlsConnection;
pub use connection::{TcpConnection, UnixConnection};
#[cfg(feature = "tls")]
pub use server::TlsServerTransport;
pub use server::{TcpServerTransport, UnixServerTransport};
#[cfg(feature = "tls")]
pub use tokio_native_tls;
