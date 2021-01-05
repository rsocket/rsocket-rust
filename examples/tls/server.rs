#[macro_use]
extern crate log;

use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::tokio_native_tls::{native_tls, TlsAcceptor};
use rsocket_rust_transport_tcp::TlsServerTransport;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_timestamp_millis().init();

    let der = include_bytes!("identity.p12");
    let cert = native_tls::Identity::from_pkcs12(der, "mypass")?;
    RSocketFactory::receive()
        .acceptor(Box::new(|setup, _socket| {
            info!("connection established: {:?}", setup);
            Ok(Box::new(EchoRSocket))
        }))
        .transport(TlsServerTransport::new(
            "127.0.0.1:4444".parse()?,
            TlsAcceptor::from(native_tls::TlsAcceptor::builder(cert).build()?),
        ))
        .serve()
        .await
}
