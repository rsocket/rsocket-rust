extern crate bytes;

use crate::frame::{Body, Frame, Writeable};
use bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct MetadataPush {
  metadata: Option<Bytes>,
}

pub struct MetadataPushBuiler {
  stream_id: u32,
  flag: u16,
  value: MetadataPush,
}

impl MetadataPushBuiler {
  fn new(stream_id: u32, flag: u16) -> MetadataPushBuiler {
    MetadataPushBuiler {
      stream_id: stream_id,
      flag: flag,
      value: MetadataPush { metadata: None },
    }
  }

  pub fn set_metadata(&mut self, metadata: Bytes) -> &mut MetadataPushBuiler {
    self.value.metadata = Some(metadata);
    self
  }

  pub fn build(&mut self) -> Frame {
    Frame::new(
      self.stream_id,
      Body::MetadataPush(self.value.clone()),
      self.flag,
    )
  }
}

impl MetadataPush {
  pub fn decode(flag: u16, bf: &mut BytesMut) -> Option<MetadataPush> {
    let m = Bytes::from(bf.to_vec());
    Some(MetadataPush { metadata: Some(m) })
  }

  pub fn builder(stream_id: u32, flag: u16) -> MetadataPushBuiler {
    MetadataPushBuiler::new(stream_id, flag)
  }

  pub fn get_metadata(&self) -> Option<Bytes> {
    self.metadata.clone()
  }

  pub fn get_data(&self) -> Option<Bytes> {
    None
  }
}

impl Writeable for MetadataPush {
  fn write_to(&self, bf: &mut BytesMut) {
    match &self.metadata {
      Some(v) => bf.put(v),
      None => (),
    }
  }

  fn len(&self) -> u32 {
    match &self.metadata {
      Some(v) => v.len() as u32,
      None => 0,
    }
  }
}
