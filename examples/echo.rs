#[macro_use]
extern crate log;
extern crate rsocket_rust;
extern crate rsocket_rust_transport_tcp;
extern crate rsocket_rust_transport_websocket;
extern crate tokio;

use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpServerTransport;
use rsocket_rust_transport_websocket::WebsocketServerTransport;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::builder().format_timestamp_millis().init();
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:7878".to_string());

    RSocketFactory::receive()
        .transport(WebsocketServerTransport::from(addr))
        .acceptor(|setup, _socket| {
            info!("accept setup: {:?}", setup);
            Ok(Box::new(EchoRSocket))
            // Or you can reject setup
            // Err(From::from("SETUP_NOT_ALLOW"))
        })
        .on_start(|| info!("+++++++ echo server started! +++++++"))
        .serve()
        .await
}
