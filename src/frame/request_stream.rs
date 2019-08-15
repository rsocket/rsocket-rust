extern crate bytes;

use crate::frame::{Body, Frame, Writeable, REQUEST_MAX};
use bytes::{BigEndian, BufMut, Bytes, BytesMut};

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
  pub fn build(&mut self) -> Frame {
    Frame::new(
      self.stream_id,
      Body::RequestStream(self.value.clone()),
      self.flag,
    )
  }

  pub fn set_initial_request_n(&mut self, n: u32) -> &mut RequestStreamBuilder {
    self.value.initial_request_n = n;
    self
  }

  pub fn set_data(&mut self, data: Bytes) -> &mut RequestStreamBuilder {
    self.value.data = Some(data);
    self
  }

  pub fn set_metadata(&mut self, metadata: Bytes) -> &mut RequestStreamBuilder {
    self.value.metadata = Some(metadata);
    self
  }
}

impl Writeable for RequestStream {
  fn write_to(&self, bf: &mut BytesMut) {
    unimplemented!()
  }

  fn len(&self) -> u32 {
    unimplemented!()
  }
}

impl RequestStream {
  
  pub fn decode(flag: u16, bf: &mut BytesMut) -> Option<RequestStream> {
    unimplemented!()
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
