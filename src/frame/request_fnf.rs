extern crate bytes;

use crate::result::RSocketResult;
use super::{Body, Frame, PayloadSupport, Writeable, FLAG_METADATA};
use bytes::{BufMut, ByteOrder, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct RequestFNF {
  metadata: Option<Bytes>,
  data: Option<Bytes>,
}

pub struct RequestFNFBuilder {
  stream_id: u32,
  flag: u16,
  value: RequestFNF,
}

impl RequestFNFBuilder {
  fn new(stream_id: u32, flag: u16) -> RequestFNFBuilder {
    RequestFNFBuilder {
      stream_id: stream_id,
      flag: flag,
      value: RequestFNF {
        metadata: None,
        data: None,
      },
    }
  }

  pub fn build(self) -> Frame {
    Frame::new(
      self.stream_id,
      Body::RequestFNF(self.value.clone()),
      self.flag,
    )
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

impl RequestFNF {
  pub fn decode(flag: u16, bf: &mut BytesMut) -> RSocketResult<RequestFNF> {
    let (m, d) = PayloadSupport::read(flag, bf);
    Ok(RequestFNF {
      metadata: m,
      data: d,
    })
  }

  pub fn builder(stream_id: u32, flag: u16) -> RequestFNFBuilder {
    RequestFNFBuilder::new(stream_id, flag)
  }

  pub fn get_metadata(&self) -> Option<Bytes> {
    self.metadata.clone()
  }

  pub fn get_data(&self) -> Option<Bytes> {
    self.data.clone()
  }
}

impl Writeable for RequestFNF {
  fn write_to(&self, bf: &mut BytesMut) {
    PayloadSupport::write(bf, &self.metadata, &self.data);
  }

  fn len(&self) -> u32 {
    PayloadSupport::len(&self.metadata, &self.data)
  }
}
