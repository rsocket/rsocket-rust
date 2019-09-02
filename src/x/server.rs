extern crate futures;
extern crate tokio;

use crate::core::{Acceptor, DuplexSocket, EmptyRSocket, RSocket};
use crate::errors::RSocketError;
use crate::payload::SetupPayload;
use crate::x::URI;
use futures::future;
use futures::prelude::*;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

fn on_setup_noop(_setup: SetupPayload, _socket: Box<dyn RSocket>) -> Box<dyn RSocket> {
  Box::new(EmptyRSocket)
}

pub struct ServerBuilder {
  uri: Option<URI>,
  on_setup: fn(SetupPayload, Box<dyn RSocket>) -> Box<dyn RSocket>,
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

  pub fn serve(self) -> impl Future<Item = (), Error = ()> {
    let uri = self.uri.clone().unwrap();
    match uri {
      URI::Tcp(v) => {
        let addr = v.parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();
        let foo = Arc::new(self.on_setup);
        let next_acceptor = move || Acceptor::Generate(foo.clone());
        listener
          .incoming()
          .map_err(|e| println!("listen error: {}", e))
          .for_each(move |socket| {
            DuplexSocket::builder()
              .set_acceptor(next_acceptor())
              .from_socket(socket);
            Ok(())
          })
        // tokio::run(server);
        // unimplemented!()
      }
      _ => unimplemented!(),
    }
  }
}
