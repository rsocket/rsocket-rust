extern crate bytes;

use crate::frame::{Body, Frame, Writeable, FLAG_METADATA};
use crate::result::RSocketResult;
use bytes::{BigEndian, BufMut, ByteOrder, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct Lease {
  ttl: u32,
  number_of_requests: u32,
  metadata: Option<Bytes>,
}

pub struct LeaseBuilder {
  stream_id: u32,
  flag: u16,
  value: Lease,
}

impl LeaseBuilder {
  fn new(stream_id: u32, flag: u16) -> LeaseBuilder {
    LeaseBuilder {
      stream_id: stream_id,
      flag: flag,
      value: Lease {
        ttl: 0,
        number_of_requests: 0,
        metadata: None,
      },
    }
  }

  pub fn set_metadata(mut self, metadata: Bytes) -> Self {
    self.value.metadata = Some(metadata);
    self.flag |= FLAG_METADATA;
    self
  }

  pub fn set_ttl(mut self, ttl: u32) -> Self {
    self.value.ttl = ttl;
    self
  }

  pub fn set_number_of_requests(mut self, n: u32) -> Self {
    self.value.number_of_requests = n;
    self
  }

  pub fn build(self) -> Frame {
    Frame::new(self.stream_id, Body::Lease(self.value.clone()), self.flag)
  }
}

impl Lease {
  pub fn decode(flag: u16, bf: &mut BytesMut) -> RSocketResult<Lease> {
    let ttl = BigEndian::read_u32(bf);
    bf.advance(4);
    let n = BigEndian::read_u32(bf);
    bf.advance(4);
    let m = if flag & FLAG_METADATA != 0 {
      Some(Bytes::from(bf.to_vec()))
    } else {
      None
    };
    Ok(Lease {
      ttl: ttl,
      number_of_requests: n,
      metadata: m,
    })
  }

  pub fn builder(stream_id: u32, flag: u16) -> LeaseBuilder {
    LeaseBuilder::new(stream_id, flag)
  }

  pub fn get_number_of_requests(&self) -> u32 {
    self.number_of_requests
  }

  pub fn get_metadata(&self) -> Option<Bytes> {
    self.metadata.clone()
  }

  pub fn get_ttl(&self) -> u32 {
    self.ttl
  }
}

impl Writeable for Lease {
  fn write_to(&self, bf: &mut BytesMut) {
    bf.put_u32_be(self.ttl);
    bf.put_u32_be(self.number_of_requests);
    match &self.metadata {
      Some(v) => bf.put(v),
      None => (),
    }
  }

  fn len(&self) -> u32 {
    8 + match &self.metadata {
      Some(v) => v.len() as u32,
      None => 0,
    }
  }
}
