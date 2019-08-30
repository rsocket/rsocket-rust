extern crate futures;

use crate::x::URI;
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
  uri: Option<URI>,
  setup: SetupPayloadBuilder,
  responder: Arc<Box<dyn RSocket>>,
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
      responder: Arc::new(Box::new(EmptyRSocket)),
      setup: SetupPayload::builder(),
    }
  }

  pub fn transport(&mut self, uri: URI) -> &mut ClientBuilder {
    self.uri = Some(uri);
    self
  }

  pub fn setup(&mut self, setup: Payload) -> &mut ClientBuilder {
    if let Some(b) = setup.data() {
      self.setup.set_data(b);
    }
    if let Some(b) = setup.metadata() {
      self.setup.set_metadata(b);
    }
    self
  }

  pub fn keepalive(
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

  pub fn mime_type(&mut self,metadata_mime_type:&str,data_mime_type:&str) -> &mut ClientBuilder{
    self
    .metadata_mime_type(metadata_mime_type)
    .data_mime_type(data_mime_type)
  }

  pub fn data_mime_type(&mut self, mime_type: &str) -> &mut ClientBuilder {
    self.setup.set_data_mime_type(String::from(mime_type));
    self
  }

  pub fn metadata_mime_type(&mut self, mime_type: &str) -> &mut ClientBuilder {
    self.setup.set_metadata_mime_type(String::from(mime_type));
    self
  }

  pub fn acceptor(&mut self,acceptor: Box<dyn RSocket>) -> &mut ClientBuilder{
    self.responder = Arc::new(acceptor);
    self
  }

  pub fn start(&mut self) -> RSocketResult<Client> {
    let acceptor = self.responder.clone();
    match &self.uri {
      Some(v) => {
        match v{
          URI::Tcp(vv) => {
            let addr = vv.parse().unwrap();
        let socket = DuplexSocket::builder()
          .set_acceptor_arc(acceptor)
          .connect(&addr);
        let setup = self.setup.build();
        socket.setup(setup).wait().unwrap();
        Ok(Client::new(socket))
          }
          _ => Err(RSocketError::from("unsupported uri")  )
        }
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
