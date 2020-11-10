#[macro_use]
extern crate log;

use rsocket_rust::prelude::*;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::TcpClientTransport;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_timestamp_millis().init();
    let client = RSocketFactory::connect()
        .transport(TcpClientTransport::from("127.0.0.1:7878"))
        .acceptor(Box::new(|| {
            // Return a responder.
            Box::new(EchoRSocket)
        }))
        .start()
        .await
        .expect("Connect failed!");

    let req = Payload::builder().set_data_utf8("Ping!").build();

    match client.request_response(req).await {
        Ok(res) => info!("{:?}", res),
        Err(e) => error!("{}", e),
    }

    Ok(())
}
