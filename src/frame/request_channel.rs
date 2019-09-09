extern crate bytes;

use super::{Body, Frame, PayloadSupport, Writeable, FLAG_METADATA, REQUEST_MAX};
use crate::result::RSocketResult;
use bytes::{BigEndian, BufMut, ByteOrder, Bytes, BytesMut};

#[derive(Debug, PartialEq)]
pub struct RequestChannel {
  initial_request_n: u32,
  metadata: Option<Bytes>,
  data: Option<Bytes>,
}

pub struct RequestChannelBuilder {
  stream_id: u32,
  flag: u16,
  value: RequestChannel,
}

impl RequestChannelBuilder {
  pub fn new(stream_id: u32, flag: u16) -> RequestChannelBuilder {
    RequestChannelBuilder {
      stream_id,
      flag,
      value: RequestChannel {
        initial_request_n: REQUEST_MAX,
        metadata: None,
        data: None,
      },
    }
  }

  pub fn build(self) -> Frame {
    Frame::new(self.stream_id, Body::RequestChannel(self.value), self.flag)
  }

  pub fn set_initial_request_n(mut self, n: u32) -> Self {
    self.value.initial_request_n = n;
    self
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
}

impl RequestChannel {
  pub fn decode(flag: u16, bf: &mut BytesMut) -> RSocketResult<RequestChannel> {
    let n = BigEndian::read_u32(bf);
    bf.advance(4);
    let (m, d) = PayloadSupport::read(flag, bf);
    Ok(RequestChannel {
      initial_request_n: n,
      metadata: m,
      data: d,
    })
  }

  pub fn builder(stream_id: u32, flag: u16) -> RequestChannelBuilder {
    RequestChannelBuilder::new(stream_id, flag)
  }

  pub fn get_initial_request_n(&self) -> u32 {
    self.initial_request_n
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

impl Writeable for RequestChannel {
  fn write_to(&self, bf: &mut BytesMut) {
    bf.put_u32_be(self.initial_request_n);
    PayloadSupport::write(bf, self.get_metadata(), self.get_data());
  }

  fn len(&self) -> usize {
    4 + PayloadSupport::len(self.get_metadata(), self.get_data())
  }
}
