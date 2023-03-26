mod codec;
mod tcp;

cfg_if! {
    if #[cfg(unix)] {
        mod uds;
    }
}

pub use tcp::TcpConnection;

cfg_if! {
    if #[cfg(unix)] {
        pub use uds::UnixConnection;
    }
}

cfg_if! {
    if #[cfg(feature = "tls")] {
        mod tls;
        pub use tls::TlsConnection;
    }
}
