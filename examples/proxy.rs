#[macro_use]
extern crate log;

use futures::executor::block_on;
use rsocket_rust::prelude::*;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_timestamp_millis().init();

    RSocketFactory::receive()
        .transport(TcpServerTransport::from("127.0.0.1:7979"))
        .acceptor(Box::new(|setup, _sending_socket| {
            info!("incoming socket: setup={:?}", setup);
            Ok(Box::new(block_on(async move {
                RSocketFactory::connect()
                    .transport(TcpClientTransport::from("127.0.0.1:7878"))
                    .acceptor(Box::new(|| Box::new(EchoRSocket)))
                    .setup(Payload::from("I'm Rust!"))
                    .start()
                    .await
                    .unwrap()
            })))
        }))
        .serve()
        .await
}
