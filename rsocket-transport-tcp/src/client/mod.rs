mod tcp;
mod uds;

pub use tcp::TcpClientTransport;

cfg_if! {
    if #[cfg(unix)] {
        pub use uds::UnixClientTransport;
    }
}

cfg_if! {
    if #[cfg(feature = "tls")] {
        mod tls;
        pub use tls::TlsClientTransport;
    }
}
