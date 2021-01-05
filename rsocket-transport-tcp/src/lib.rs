#![allow(clippy::type_complexity)]

#[macro_use]
extern crate log;

mod client;
mod connection;
mod misc;
mod server;

pub use client::{TcpClientTransport, UnixClientTransport};
pub use connection::{TcpConnection, UnixConnection};
pub use server::{TcpServerTransport, UnixServerTransport};

#[cfg(feature = "tls")]
pub use client::TlsClientTransport;
#[cfg(feature = "tls")]
pub use connection::TlsConnection;
#[cfg(feature = "tls")]
pub use server::TlsServerTransport;
#[cfg(feature = "tls")]
pub use tokio_native_tls;
