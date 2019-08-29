extern crate futures;

use crate::core::{DuplexSocket, EmptyRSocket, RSocket};
use crate::errors::RSocketError;
use crate::payload::{Payload, SetupPayload, SetupPayloadBuilder};
use crate::result::RSocketResult;
use futures::{Future, Stream};
use std::sync::Arc;
use std::time::Duration;

pub struct Client {
  socket: DuplexSocket,
}

pub struct ClientBuilder {
  uri: Option<String>,
  setup: SetupPayloadBuilder,
  acceptor: Arc<Box<dyn RSocket>>,
}

impl Client {
  fn new(socket: DuplexSocket) -> Client {
    Client { socket }
  }

  pub fn builder() -> ClientBuilder {
    ClientBuilder::new()
  }
}

impl ClientBuilder {
  fn new() -> ClientBuilder {
    ClientBuilder {
      uri: None,
      acceptor: Arc::new(Box::new(EmptyRSocket)),
      setup: SetupPayload::builder(),
    }
  }

  pub fn set_uri(&mut self, uri: &str) -> &mut ClientBuilder {
    self.uri = Some(String::from(uri));
    self
  }

  pub fn set_setup(&mut self, setup: Payload) -> &mut ClientBuilder {
    if let Some(b) = setup.data() {
      self.setup.set_data(b);
    }
    if let Some(b) = setup.metadata() {
      self.setup.set_metadata(b);
    }
    self
  }

  pub fn set_keepalive(
    &mut self,
    tick_period: Duration,
    ack_timeout: Duration,
    missed_acks: u64,
  ) -> &mut ClientBuilder {
    self
      .setup
      .set_keepalive(tick_period, ack_timeout, missed_acks);
    self
  }

  pub fn set_data_mime_type(&mut self, mime_type: &str) -> &mut ClientBuilder {
    self.setup.set_data_mime_type(String::from(mime_type));
    self
  }

  pub fn set_metadata_mime_type(&mut self, mime_type: &str) -> &mut ClientBuilder {
    self.setup.set_metadata_mime_type(String::from(mime_type));
    self
  }

  pub fn build(&mut self) -> RSocketResult<Client> {
    let acceptor = self.acceptor.clone();
    match &self.uri {
      Some(v) => {
        let addr = v.parse().unwrap();
        let socket = DuplexSocket::builder()
          .set_acceptor_arc(acceptor)
          .connect(&addr);
        let setup = self.setup.build();
        socket.setup(setup).wait().unwrap();
        Ok(Client::new(socket))
      }
      None => Err(RSocketError::from("missing rsocket uri")),
    }
  }
}

impl RSocket for Client {
  fn metadata_push(&self, req: Payload) -> Box<dyn Future<Item = (), Error = RSocketError>> {
    self.socket.metadata_push(req)
  }

  fn request_fnf(&self, req: Payload) -> Box<dyn Future<Item = (), Error = RSocketError>> {
    self.socket.request_fnf(req)
  }

  fn request_response(
    &self,
    req: Payload,
  ) -> Box<dyn Future<Item = Payload, Error = RSocketError>> {
    self.socket.request_response(req)
  }

  fn request_stream(&self, req: Payload) -> Box<dyn Stream<Item = Payload, Error = RSocketError>> {
    self.socket.request_stream(req)
  }
}
