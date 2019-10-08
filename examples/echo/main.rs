extern crate rsocket_rust;
extern crate tokio;
#[macro_use]
extern crate log;
use futures::{SinkExt, StreamExt};
use rsocket_rust::frame::Frame;
use rsocket_rust::prelude::*;
use rsocket_rust::transport::DuplexSocket;
use rsocket_rust::transport::{self, Rx, Tx};
use std::env;
use std::error::Error;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::prelude::*;
use tokio::sync::mpsc;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().init();
    RSocketFactory::receive()
        .transport(URI::Tcp("127.0.0.1:7878"))
        .acceptor(|setup, sending_socket| {
            info!("accept setup: {:?}", setup);
            Box::new(EchoRSocket)
        })
        .serve()
        .await

    // let addr = env::args().nth(1).unwrap_or("127.0.0.1:7878".to_string());
    // let mut listener = TcpListener::bind(&addr).await?;
    // println!("Listening on: {}", addr);

    // loop {
    //     let (mut socket, _) = listener.accept().await?;
    //     let (rcv_tx, mut rcv_rx) = mpsc::unbounded_channel::<Frame>();
    //     let (snd_tx, snd_rx) = mpsc::unbounded_channel::<Frame>();

    //     tokio::spawn(
    //         async move { rsocket_rust::transport::tcp::process(socket, snd_rx, rcv_tx).await },
    //     );

    //     let ds = DuplexSocket::new(0, snd_tx.clone());
    //     tokio::spawn(async move {
    //         let acceptor = Acceptor::Generate(Arc::new(|setup, socket| Box::new(EchoRSocket)));
    //         ds.event_loop(acceptor, rcv_rx).await;
    //     });
    // }
}
