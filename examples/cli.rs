#[macro_use]
extern crate log;

use futures::channel::mpsc;
use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpClientTransport;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::builder().format_timestamp_millis().init();

    let (mut tx, mut rx) = mpsc::channel::<u8>(1);
    let cli = RSocketFactory::connect()
        .transport(TcpClientTransport::from("127.0.0.1:7878"))
        .on_close(Box::new(move || {
            tx.try_send(1).unwrap();
        }))
        .start()
        .await
        .expect("Connect failed!");
    let req = Payload::builder().set_data_utf8("Bingo!").build();
    let res = cli.request_response(req).await.expect("Request failed!");

    info!("----> rcv response: {:?}", res);
    rx.next().await;
    info!("socket closed!");

    Ok(())
}
