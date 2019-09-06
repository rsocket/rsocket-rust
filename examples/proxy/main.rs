#[macro_use]
extern crate log;
extern crate rsocket_rust;
extern crate tokio;

use rsocket_rust::prelude::*;

fn main() {
  env_logger::builder()
    .default_format_timestamp_nanos(true)
    .init();
  let server = RSocketFactory::receive()
    .acceptor(|setup, _sending_socket| {
      info!("incoming socket: setup={:?}", setup);
      proxied(URI::Tcp("127.0.0.1:7878"))
    })
    .transport(URI::Tcp("127.0.0.1:7979"))
    .serve();
  tokio::run(server);
}

fn proxied(target: URI) -> Box<dyn RSocket> {
  Box::new(
    RSocketFactory::connect()
      .acceptor(|| Box::new(MockResponder))
      .setup(Payload::from("I'm Rust!"))
      .transport(target)
      .start()
      .unwrap(),
  )
}
