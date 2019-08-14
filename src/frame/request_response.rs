extern crate bytes;

use crate::frame::{Body, Frame, Writeable, FLAG_METADATA, U24};
use bytes::{BigEndian, BufMut, Bytes, BytesMut};

#[derive(Debug, Clone)]
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
      stream_id: stream_id,
      flag: flag,
      value: RequestResponse {
        metadata: None,
        data: None,
      },
    }
  }

  pub fn set_metadata(&mut self, metadata: Bytes) -> &mut RequestResponseBuilder {
    self.value.metadata = Some(metadata);
    self.flag |= FLAG_METADATA;
    self
  }

  pub fn set_data(&mut self, data: Bytes) -> &mut RequestResponseBuilder {
    self.value.data = Some(data);
    self
  }

  pub fn build(&mut self) -> Frame {
    Frame::new(
      self.stream_id,
      Body::RequestResponse(self.value.clone()),
      self.flag,
    )
  }
}

impl Writeable for RequestResponse {
  fn write_to(&self, bf: &mut BytesMut) {
    match &self.metadata {
      Some(v) => {
        U24::write(v.len() as u32, bf);
        bf.put(v);
      }
      None => (),
    }
    match &self.data {
      Some(v) => bf.put(v),
      None => (),
    }
  }

  fn len(&self) -> u32 {
    let a: u32 = match &self.metadata {
      Some(v) => 3 + (v.len() as u32),
      None => 0,
    };
    let b: u32 = match &self.data {
      Some(v) => v.len() as u32,
      None => 0,
    };
    a + b
  }
}

impl RequestResponse {
  pub fn decode(flag: u16, bf: &mut BytesMut) -> Option<RequestResponse> {
    let m: Option<Bytes> = if flag & FLAG_METADATA != 0 {
      let n = U24::read(bf);
      Some(Bytes::from(bf.split_to(n as usize)))
    } else {
      None
    };
    let d: Option<Bytes> = if bf.is_empty() {
      None
    } else {
      Some(Bytes::from(bf.to_vec()))
    };
    Some(RequestResponse {
      metadata: m,
      data: d,
    })
  }

  pub fn builder(stream_id: u32, flag: u16) -> RequestResponseBuilder {
    RequestResponseBuilder::new(stream_id, flag)
  }

  pub fn get_metadata(&self) -> Option<Bytes> {
    self.metadata.clone()
  }

  pub fn get_data(&self) -> Option<Bytes> {
    self.data.clone()
  }
}
