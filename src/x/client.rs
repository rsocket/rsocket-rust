extern crate futures;
extern crate tokio;

use super::URI;
use crate::core::{Acceptor, DuplexSocket, RSocket};
use crate::errors::RSocketError;
use crate::payload::{Payload, SetupPayload, SetupPayloadBuilder};
use crate::result::RSocketResult;
use futures::{Future, Stream};

use std::time::Duration;
use tokio::runtime::Runtime;

pub struct Client {
  socket: DuplexSocket,
  rt: Runtime,
}

pub struct ClientBuilder {
  uri: Option<URI>,
  setup: SetupPayloadBuilder,
  responder: Option<fn() -> Box<dyn RSocket>>,
}

impl Client {
  fn new(socket: DuplexSocket, rt: Runtime) -> Client {
    Client {
      socket: socket,
      rt: rt,
    }
  }

  pub fn on_close(self) -> impl Future<Item = (), Error = ()> {
    self.rt.shutdown_on_idle()
  }

  pub fn builder() -> ClientBuilder {
    ClientBuilder::new()
  }
}

impl ClientBuilder {
  fn new() -> ClientBuilder {
    ClientBuilder {
      uri: None,
      responder: None,
      setup: SetupPayload::builder(),
    }
  }

  pub fn transport(mut self, uri: URI) -> Self {
    self.uri = Some(uri);
    self
  }

  pub fn setup(mut self, setup: Payload) -> Self {
    if let Some(b) = setup.data() {
      self.setup = self.setup.set_data(b);
    }
    if let Some(b) = setup.metadata() {
      self.setup = self.setup.set_metadata(b);
    }
    self
  }

  pub fn keepalive(
    mut self,
    tick_period: Duration,
    ack_timeout: Duration,
    missed_acks: u64,
  ) -> Self {
    self.setup = self
      .setup
      .set_keepalive(tick_period, ack_timeout, missed_acks);
    self
  }

  pub fn mime_type(mut self, metadata_mime_type: &str, data_mime_type: &str) -> Self {
    self = self.metadata_mime_type(metadata_mime_type);
    self = self.data_mime_type(data_mime_type);
    self
  }

  pub fn data_mime_type(mut self, mime_type: &str) -> Self {
    self.setup = self.setup.set_data_mime_type(mime_type);
    self
  }

  pub fn metadata_mime_type(mut self, mime_type: &str) -> Self {
    self.setup = self.setup.set_metadata_mime_type(mime_type);
    self
  }

  pub fn acceptor(mut self, acceptor: fn() -> Box<dyn RSocket>) -> Self {
    self.responder = Some(acceptor);
    self
  }

  pub fn start(self) -> RSocketResult<Client> {
    match self.uri {
      Some(v) => match v {
        URI::Tcp(vv) => {
          let addr = vv.parse().unwrap();
          let mut bu = DuplexSocket::builder();
          if let Some(r) = self.responder {
            bu = bu.set_acceptor(Acceptor::Direct(r()));
          }
          let (socket, daemon) = bu.connect(&addr);
          let mut rt = Runtime::new().unwrap();
          rt.spawn(daemon);
          socket.setup(self.setup.build()).wait().unwrap();
          Ok(Client::new(socket, rt))
        }
        _ => Err(RSocketError::from("unsupported uri")),
      },
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
