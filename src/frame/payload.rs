extern crate bytes;

use crate::result::RSocketResult;
use crate::frame::{Body, Frame, PayloadSupport, Writeable, FLAG_METADATA};
use bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct Payload {
  metadata: Option<Bytes>,
  data: Option<Bytes>,
}

pub struct PayloadBuilder {
  stream_id: u32,
  flag: u16,
  value: Payload,
}

impl PayloadBuilder {
  fn new(stream_id: u32, flag: u16) -> PayloadBuilder {
    PayloadBuilder {
      stream_id: stream_id,
      flag: flag,
      value: Payload {
        metadata: None,
        data: None,
      },
    }
  }

  pub fn set_data(&mut self, data: Bytes) -> &mut PayloadBuilder {
    self.value.data = Some(data);
    self
  }

  pub fn set_metadata(&mut self, metadata: Bytes) -> &mut PayloadBuilder {
    self.value.metadata = Some(metadata);
    self.flag |= FLAG_METADATA;
    self
  }

  pub fn build(&mut self) -> Frame {
    Frame::new(self.stream_id, Body::Payload(self.value.clone()), self.flag)
  }
}

impl Payload {
  pub fn decode(flag: u16, bf: &mut BytesMut) -> RSocketResult<Payload> {
    let (m, d) = PayloadSupport::read(flag, bf);
    Ok(Payload {
      metadata: m,
      data: d,
    })
  }

  pub fn builder(stream_id: u32, flag: u16) -> PayloadBuilder {
    PayloadBuilder::new(stream_id, flag)
  }

  pub fn get_metadata(&self) -> Option<Bytes> {
    self.metadata.clone()
  }

  pub fn get_data(&self) -> Option<Bytes> {
    self.data.clone()
  }
}

impl Writeable for Payload {
  fn write_to(&self, bf: &mut BytesMut) {
    PayloadSupport::write(bf, &self.metadata, &self.data);
  }

  fn len(&self) -> u32 {
    PayloadSupport::len(&self.metadata, &self.data)
  }
}
