extern crate rsocket_rust;
extern crate tokio;
#[macro_use]
extern crate log;
use rsocket_rust::prelude::*;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().format_timestamp_millis().init();
    let addr = env::args()
        .nth(1)
        .unwrap_or("tcp://127.0.0.1:7878".to_string());
    RSocketFactory::receive()
        .transport(&addr)
        .acceptor(|setup, _socket| {
            info!("accept setup: {:?}", setup);
            Box::new(EchoRSocket)
        })
        .on_start(|| info!("+++++++ echo server started! +++++++"))
        .serve()
        .await
}
