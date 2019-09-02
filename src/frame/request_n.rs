extern crate bytes;

use crate::result::RSocketResult;
use crate::frame::{Body, Frame, Writeable, REQUEST_MAX};
use bytes::{BigEndian, BufMut, ByteOrder, BytesMut};

#[derive(Debug, Clone)]
pub struct RequestN {
  n: u32,
}

pub struct RequestNBuilder {
  stream_id: u32,
  flag: u16,
  value: RequestN,
}

impl RequestNBuilder {
  fn new(stream_id: u32, flag: u16) -> RequestNBuilder {
    RequestNBuilder {
      stream_id: stream_id,
      flag: flag,
      value: RequestN { n: REQUEST_MAX },
    }
  }

  pub fn set_n(mut self, n: u32) -> Self {
    self.value.n = n;
    self
  }

  pub fn build(self) -> Frame {
    Frame::new(
      self.stream_id,
      Body::RequestN(self.value.clone()),
      self.flag,
    )
  }
}

impl RequestN {
  pub fn decode(flag: u16, bf: &mut BytesMut) -> RSocketResult<RequestN> {
    let n = BigEndian::read_u32(bf);
    bf.advance(4);
    Ok(RequestN { n: n })
  }

  pub fn builder(stream_id: u32, flag: u16) -> RequestNBuilder {
    RequestNBuilder::new(stream_id, flag)
  }

  pub fn get_n(&self) -> u32 {
    self.n.clone()
  }
}

impl Writeable for RequestN {
  fn write_to(&self, bf: &mut BytesMut) {
    bf.put_u32_be(self.n)
  }

  fn len(&self) -> u32 {
    4
  }
}
