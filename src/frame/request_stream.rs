extern crate bytes;

use crate::result::RSocketResult;
use super::{Body, Frame, PayloadSupport, Writeable, FLAG_METADATA, REQUEST_MAX, U24};
use bytes::{BigEndian, BufMut, ByteOrder, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct RequestStream {
  initial_request_n: u32,
  metadata: Option<Bytes>,
  data: Option<Bytes>,
}

pub struct RequestStreamBuilder {
  stream_id: u32,
  flag: u16,
  value: RequestStream,
}

impl RequestStreamBuilder {
  pub fn build(self) -> Frame {
    Frame::new(
      self.stream_id,
      Body::RequestStream(self.value.clone()),
      self.flag,
    )
  }

  pub fn set_initial_request_n(mut self, n: u32) -> Self {
    self.value.initial_request_n = n;
    self
  }

  pub fn set_data(mut self, data: Bytes) -> Self {
    self.value.data = Some(data);
    self
  }

  pub fn set_metadata(mut self, metadata: Bytes) -> Self {
    self.value.metadata = Some(metadata);
    self.flag |= FLAG_METADATA;
    self
  }
}

impl Writeable for RequestStream {
  fn write_to(&self, bf: &mut BytesMut) {
    bf.put_u32_be(self.initial_request_n);
    PayloadSupport::write(bf, &self.metadata, &self.data)
  }

  fn len(&self) -> u32 {
    4 + PayloadSupport::len(&self.metadata, &self.data)
  }
}

impl RequestStream {

  pub fn decode(flag: u16, bf: &mut BytesMut) -> RSocketResult<RequestStream> {
    let n = BigEndian::read_u32(bf);
    bf.advance(4);
    let (m, d) = PayloadSupport::read(flag, bf);
    Ok(RequestStream {
      initial_request_n: n,
      metadata: m,
      data: d,
    })
  }

  pub fn builder(stream_id: u32, flag: u16) -> RequestStreamBuilder {
    RequestStreamBuilder {
      stream_id: stream_id,
      flag: flag,
      value: RequestStream {
        initial_request_n: REQUEST_MAX,
        metadata: None,
        data: None,
      },
    }
  }

  pub fn get_initial_request_n(&self) -> u32 {
    self.initial_request_n
  }

  pub fn get_metadata(&self) -> Option<Bytes> {
    self.metadata.clone()
  }

  pub fn get_data(&self) -> Option<Bytes> {
    self.data.clone()
  }
}
