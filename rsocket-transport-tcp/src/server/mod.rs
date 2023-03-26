mod tcp;

cfg_if! {
    if #[cfg(unix)] {
        mod uds;
    }
}

pub use tcp::TcpServerTransport;

cfg_if! {
    if #[cfg(unix)] {
        pub use uds::UnixServerTransport;
    }
}

cfg_if! {
    if #[cfg(feature = "tls")] {
        mod tls;
        pub use tls::TlsServerTransport;
    }
}
