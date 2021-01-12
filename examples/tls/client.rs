#[macro_use]
extern crate log;

use rsocket_rust::prelude::*;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::tokio_native_tls::{native_tls, TlsConnector};
use rsocket_rust_transport_tcp::TlsClientTransport;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_timestamp_millis().init();

    let pem = include_bytes!("foobar.com.pem");
    let cert = native_tls::Certificate::from_pem(pem)?;
    let cx = native_tls::TlsConnector::builder()
        .add_root_certificate(cert)
        .build()?;
    let cx = TlsConnector::from(cx);
    let cli = RSocketFactory::connect()
        .transport(TlsClientTransport::new(
            "foobar.com".into(),
            "127.0.0.1:4444".parse()?,
            cx,
        ))
        .start()
        .await?;
    let res = cli
        .request_response(Payload::builder().set_data_utf8("hello").build())
        .await?;
    info!("response: {:?}", res);

    cli.wait_for_close().await;

    Ok(())
}
