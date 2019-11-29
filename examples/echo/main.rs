extern crate rsocket_rust;
extern crate tokio;
#[macro_use]
extern crate log;
use rsocket_rust::prelude::*;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().init();
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:7878".to_string());

    RSocketFactory::receive()
        .transport(URI::Tcp(addr))
        .acceptor(|setup, _socket| {
            info!("accept setup: {:?}", setup);
            Box::new(EchoRSocket)
        })
        .serve()
        .await
}
