use tokio::codec::Framed;
use tokio::codec::LinesCodec;
use tokio::net::TcpListener;
use tokio::prelude::*;

#[test]
fn test_serve() {




let addr = "127.0.0.1:8787".parse().unwrap();
  let listener = TcpListener::bind(&addr).unwrap();
  let server = listener
    .incoming()
    .for_each(|socket| {
      let framed_sock = Framed::new(socket, LinesCodec::new());
      framed_sock.for_each(|line| {
        println!("Received line {}", line);
        Ok(())
      })
    })
    .map_err(|err| println!("error: {:?}", err));
  tokio::run(server);
}
