extern crate bytes;

use crate::frame::{Body, Frame, PayloadSupport, Writeable, FLAG_METADATA, REQUEST_MAX};
use bytes::{BigEndian, BufMut, ByteOrder, Bytes, BytesMut};

#[derive(Debug, Clone)]
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
      stream_id: stream_id,
      flag: flag,
      value: RequestChannel {
        initial_request_n: REQUEST_MAX,
        metadata: None,
        data: None,
      },
    }
  }

  pub fn build(&mut self) -> Frame {
    Frame::new(
      self.stream_id,
      Body::RequestChannel(self.value.clone()),
      self.flag,
    )
  }

  pub fn set_initial_request_n(&mut self, n: u32) -> &mut RequestChannelBuilder {
    self.value.initial_request_n = n;
    self
  }

  pub fn set_metadata(&mut self, metadata: Bytes) -> &mut RequestChannelBuilder {
    self.value.metadata = Some(metadata);
    self.flag |= FLAG_METADATA;
    self
  }

  pub fn set_data(&mut self, data: Bytes) -> &mut RequestChannelBuilder {
    self.value.data = Some(data);
    self
  }
}

impl RequestChannel {
  pub fn decode(flag: u16, bf: &mut BytesMut) -> Option<RequestChannel> {
    let n = BigEndian::read_u32(bf);
    bf.advance(4);
    let (m, d) = PayloadSupport::read(flag, bf);
    Some(RequestChannel {
      initial_request_n: n,
      metadata: m,
      data: d,
    })
  }

  pub fn builder(stream_id: u32, flag: u16) -> RequestChannelBuilder {
    RequestChannelBuilder::new(stream_id, flag)
  }

  pub fn get_initial_request_n(&self) -> u32 {
    self.initial_request_n.clone()
  }

  pub fn get_metadata(&self) -> Option<Bytes> {
    self.metadata.clone()
  }
  pub fn get_data(&self) -> Option<Bytes> {
    self.data.clone()
  }
}

impl Writeable for RequestChannel {
  fn write_to(&self, bf: &mut BytesMut) {
    bf.put_u32_be(self.initial_request_n);
    PayloadSupport::write(bf, &self.metadata, &self.data);
  }

  fn len(&self) -> u32 {
    4 + PayloadSupport::len(&self.metadata, &self.data)
  }
}
