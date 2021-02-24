#![allow(clippy::type_complexity)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate cfg_if;

mod client;
mod connection;
mod misc;
mod server;

pub use client::{TcpClientTransport, UnixClientTransport};
pub use connection::{TcpConnection, UnixConnection};
pub use server::{TcpServerTransport, UnixServerTransport};

cfg_if! {
    if #[cfg(feature = "tls")] {
        pub use tokio_native_tls;
        pub use client::TlsClientTransport;
        pub use connection::TlsConnection;
        pub use server::TlsServerTransport;
    }
}
