extern crate futures;

use crate::payload::Payload;
use crate::result::Result;

pub trait RSocket {
  fn request_response(&self, msg: Payload) -> Result<Payload>;
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

  pub fn request_response(&self, msg: Payload) -> Result<Payload> {
    self.hooks.request_response(msg)
  }
}
