extern crate futures;

use crate::errors::RSocketError;
use crate::payload::Payload;
use futures::{Future, Stream};

pub trait RSocket {
  fn metadata_push(&self,req:Payload) -> Box<Future<Item=(),Error=RSocketError>>;
  fn request_fnf(&self,req:Payload) -> Box<Future<Item=(),Error=RSocketError>>;
  fn request_response(&self, req: Payload) -> Box<Future<Item = Payload, Error = RSocketError>>;
  fn request_stream(&self, req: Payload) -> Box<Stream<Item = Payload, Error = RSocketError>>;
}

pub struct Responder {
  hooks: Box<RSocket>,
}

impl Responder {
  pub fn new<E: RSocket + 'static>(hooks: E) -> Self {
    Self {
      hooks: Box::new(hooks),
    }
  }
}
