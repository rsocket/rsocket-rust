extern crate bytes;

use crate::frame::{Body, Frame, Writeable};
use bytes::{BigEndian, BufMut, ByteOrder, Bytes, BytesMut};



#[derive(Debug, Clone)]
pub struct Error {
  code: u32,
  data: Option<Bytes>,
}

pub struct ErrorBuilder {
  stream_id: u32,
  flag: u16,
  value: Error,
}

impl ErrorBuilder {
  fn new(stream_id: u32, flag: u16) -> ErrorBuilder {
    ErrorBuilder {
      stream_id: stream_id,
      flag: flag,
      value: Error {
        code: 0,
        data: None,
      },
    }
  }

  pub fn set_code(&mut self, code: u32) -> &mut ErrorBuilder {
    self.value.code = code;
    self
  }

  pub fn set_data(&mut self, data: Bytes) -> &mut ErrorBuilder {
    self.value.data = Some(data);
    self
  }

  pub fn build(&mut self) -> Frame {
    Frame::new(self.stream_id, Body::Error(self.value.clone()), self.flag)
  }
}

impl Error {
  pub fn decode(flag: u16, bf: &mut BytesMut) -> Option<Error> {
    let code = BigEndian::read_u32(bf);
    bf.advance(4);
    let d: Option<Bytes> = if bf.is_empty() {
      None
    } else {
      Some(Bytes::from(bf.to_vec()))
    };
    Some(Error {
      code: code,
      data: d,
    })
  }

  pub fn builder(stream_id: u32, flag: u16) -> ErrorBuilder {
    ErrorBuilder::new(stream_id, flag)
  }

  pub fn get_data(&self) -> Option<Bytes> {
    self.data.clone()
  }

  pub fn get_code(&self) -> u32 {
    self.code
  }
}

impl Writeable for Error {
  fn write_to(&self, bf: &mut BytesMut) {
    bf.put_u32_be(self.code);
    match &self.data {
      Some(v) => bf.put(v),
      None => (),
    }
  }

  fn len(&self) -> u32 {
    4 + match &self.data {
      Some(v) => v.len() as u32,
      None => 0,
    }
  }
}
