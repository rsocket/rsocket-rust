extern crate futures;
extern crate tokio;

use crate::core::{DuplexSocket, EmptyRSocket, RSocket};
use crate::errors::RSocketError;
use crate::x::URI;
use futures::future;
use futures::prelude::*;
use std::net::SocketAddr;
use std::sync::Arc;
use tokio::net::{TcpListener, TcpStream};

pub struct Server {}

pub struct ServerBuilder {
  uri: Option<URI>,
  responder: Vec<Box<dyn RSocket>>,
}

impl ServerBuilder {
  pub fn new() -> ServerBuilder {
    ServerBuilder {
      uri: None,
      responder: vec![],
    }
  }

  pub fn acceptor(&mut self, acceptor: Box<dyn RSocket>) -> &mut ServerBuilder {
    self.responder.push(acceptor);
    self
  }

  pub fn transport(&mut self, uri: URI) -> &mut ServerBuilder {
    self.uri = Some(uri);
    self
  }

  pub fn serve(&mut self) -> impl Future<Item = (), Error = ()> {
    let uri = self.uri.clone().unwrap();
    match uri {
      URI::Tcp(v) => {
        let acceptor = self.responder.remove(0);
        let addr = v.parse().unwrap();
        let listener = TcpListener::bind(&addr).unwrap();
        let foo = Arc::new(acceptor);
        let xx = move || foo.clone();
        listener
          .incoming()
          .map_err(|e| println!("listen error: {}", e))
          .for_each(move |socket| {
            let sk = DuplexSocket::builder()
              .set_acceptor_arc(xx())
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
