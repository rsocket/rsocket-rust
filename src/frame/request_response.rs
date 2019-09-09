extern crate bytes;

use super::{Body, Frame, PayloadSupport, Writeable, FLAG_METADATA, U24};
use crate::result::RSocketResult;
use bytes::{BigEndian, BufMut, Bytes, BytesMut};

#[derive(Debug, PartialEq)]
pub struct RequestResponse {
  metadata: Option<Bytes>,
  data: Option<Bytes>,
}

pub struct RequestResponseBuilder {
  stream_id: u32,
  flag: u16,
  value: RequestResponse,
}

impl RequestResponseBuilder {
  fn new(stream_id: u32, flag: u16) -> RequestResponseBuilder {
    RequestResponseBuilder {
      stream_id,
      flag,
      value: RequestResponse {
        metadata: None,
        data: None,
      },
    }
  }

  pub fn set_metadata(mut self, metadata: Bytes) -> Self {
    self.value.metadata = Some(metadata);
    self.flag |= FLAG_METADATA;
    self
  }

  pub fn set_data(mut self, data: Bytes) -> Self {
    self.value.data = Some(data);
    self
  }

  pub fn build(self) -> Frame {
    Frame::new(self.stream_id, Body::RequestResponse(self.value), self.flag)
  }
}

impl RequestResponse {
  pub fn decode(flag: u16, bf: &mut BytesMut) -> RSocketResult<RequestResponse> {
    let (m, d) = PayloadSupport::read(flag, bf);
    Ok(RequestResponse {
      metadata: m,
      data: d,
    })
  }

  pub fn builder(stream_id: u32, flag: u16) -> RequestResponseBuilder {
    RequestResponseBuilder::new(stream_id, flag)
  }

  pub fn get_metadata(&self) -> &Option<Bytes> {
    &self.metadata
  }

  pub fn get_data(&self) -> &Option<Bytes> {
    &self.data
  }

  pub fn split(self) -> (Option<Bytes>, Option<Bytes>) {
    (self.data, self.metadata)
  }
}

impl Writeable for RequestResponse {
  fn write_to(&self, bf: &mut BytesMut) {
    PayloadSupport::write(bf, self.get_metadata(), self.get_data())
  }

  fn len(&self) -> usize {
    PayloadSupport::len(self.get_metadata(), self.get_data())
  }
}
