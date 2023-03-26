#![allow(clippy::type_complexity)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate cfg_if;

mod client;
mod connection;
mod misc;
mod server;

pub use client::TcpClientTransport;
pub use connection::TcpConnection;
pub use server::TcpServerTransport;

cfg_if! {
    if #[cfg(unix)]{
        pub use client::UnixClientTransport;
        pub use connection::UnixConnection;
        pub use server::UnixServerTransport;
    }
}

cfg_if! {
    if #[cfg(feature = "tls")] {
        pub use tokio_native_tls;
        pub use client::TlsClientTransport;
        pub use connection::TlsConnection;
        pub use server::TlsServerTransport;
    }
}
