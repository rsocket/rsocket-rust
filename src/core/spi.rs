extern crate bytes;
extern crate futures;

use crate::errors::RSocketError;
use crate::payload::Payload;
use bytes::Bytes;
use futures::future::ok;
use futures::stream::iter_ok;
use futures::{Future, Stream};

pub trait RSocket: Sync + Send {
  fn metadata_push(&self, req: Payload) -> Box<dyn Future<Item = (), Error = RSocketError>>;
  fn fire_and_forget(&self, req: Payload) -> Box<dyn Future<Item = (), Error = RSocketError>>;
  fn request_response(&self, req: Payload)
    -> Box<dyn Future<Item = Payload, Error = RSocketError>>;
  fn request_stream(&self, req: Payload) -> Box<dyn Stream<Item = Payload, Error = RSocketError>>;
}

pub struct MockResponder;

impl RSocket for MockResponder {
  fn metadata_push(&self, req: Payload) -> Box<dyn Future<Item = (), Error = RSocketError>> {
    debug!("receive metadata_push: {:?}", req);
    Box::new(ok(()).map_err(|()| RSocketError::from("foobar")))
  }

  fn fire_and_forget(&self, req: Payload) -> Box<dyn Future<Item = (), Error = RSocketError>> {
    debug!("receive request_fnf: {:?}", req);
    Box::new(ok(()).map_err(|()| RSocketError::from("foobar")))
  }

  fn request_response(
    &self,
    req: Payload,
  ) -> Box<dyn Future<Item = Payload, Error = RSocketError>> {
    debug!(">>>>>>>> mock responder: {:?}", req);
    Box::new(ok(req))
  }

  fn request_stream(&self, req: Payload) -> Box<dyn Stream<Item = Payload, Error = RSocketError>> {
    debug!(">>>>>>>> accept stream: {:?}", req);
    let mut results = vec![];
    for n in 0..10 {
      let pa = Payload::builder()
        .set_data(Bytes::from(format!("DATA_{}", n)))
        .set_metadata(Bytes::from(format!("METADATA_{}", n)))
        .build();
      results.push(pa);
    }
    Box::new(iter_ok(results))
  }
}
