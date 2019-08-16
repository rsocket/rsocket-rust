extern crate rsocket;
extern crate tokio;

use rsocket::transport::*;
use tokio::codec::Framed;
use tokio::net::TcpListener;
use tokio::prelude::*;

// #[test]
fn test_serve() {
  // serve and echo each incoming frame.
  let addr = "127.0.0.1:7878".parse().unwrap();
  let listener = TcpListener::bind(&addr).unwrap();
  let server = listener
    .incoming()
    .for_each(|socket| {
      let framed_sock = Framed::new(socket, FrameCodec::new());
      framed_sock.for_each(|f| {
        println!("received frame {:?}", f);
        Ok(())
      })
    })
    .map_err(|err| println!("error: {:?}", err));
  tokio::run(server);
}
