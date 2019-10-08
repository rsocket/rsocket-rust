use super::URI;
use crate::frame::{self, Frame};
use crate::payload::SetupPayload;
use crate::spi::{Acceptor, EmptyRSocket, RSocket};
use crate::transport::DuplexSocket;
use std::error::Error;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::TcpListener;
use tokio::sync::mpsc;

fn on_setup_noop(_setup: SetupPayload, _socket: Box<dyn RSocket>) -> Box<dyn RSocket> {
  Box::new(EmptyRSocket)
}

pub struct ServerBuilder {
  uri: Option<URI>,
  on_setup: fn(SetupPayload, Box<dyn RSocket>) -> Box<dyn RSocket>,
}

impl Default for ServerBuilder {
  fn default() -> Self {
    Self::new()
  }
}

impl ServerBuilder {
  pub fn new() -> ServerBuilder {
    ServerBuilder {
      uri: None,
      on_setup: on_setup_noop,
    }
  }

  pub fn acceptor(
    mut self,
    handler: fn(SetupPayload, Box<dyn RSocket>) -> Box<dyn RSocket>,
  ) -> Self {
    self.on_setup = handler;
    self
  }

  pub fn transport(mut self, uri: URI) -> Self {
    self.uri = Some(uri);
    self
  }

  pub async fn serve(self) -> Result<(), Box<dyn Error>> {
    let uri = self.uri.clone().unwrap();
    match uri {
      URI::Tcp(v) => {
        let addr: SocketAddr = v.parse().unwrap();
        let mut listener = TcpListener::bind(&addr).await.unwrap();
        println!("Listening on: {}", addr);
        loop {
          let (mut socket, _) = listener.accept().await.unwrap();
          let (rcv_tx, mut rcv_rx) = mpsc::unbounded_channel::<Frame>();
          let (snd_tx, snd_rx) = mpsc::unbounded_channel::<Frame>();

          tokio::spawn(async move { crate::transport::tcp::process(socket, snd_rx, rcv_tx).await });

          let setuper = Arc::new(self.on_setup);
          let next_acceptor = move || Acceptor::Generate(setuper.clone());
          let ds = DuplexSocket::new(0, snd_tx.clone());
          tokio::spawn(async move {
            let acceptor = next_acceptor();
            ds.event_loop(acceptor, rcv_rx).await;
          });
        }
      }
      _ => unimplemented!(),
    }
  }
}
