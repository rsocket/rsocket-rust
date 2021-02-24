mod codec;
mod tcp;
mod uds;

pub use tcp::TcpConnection;
pub use uds::UnixConnection;

cfg_if! {
    if #[cfg(feature = "tls")] {
        mod tls;
        pub use tls::TlsConnection;
    }
}
