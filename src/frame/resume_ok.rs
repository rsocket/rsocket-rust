extern crate bytes;

use crate::result::RSocketResult;
use crate::frame::{Body, Frame, Writeable};
use bytes::{BigEndian, BufMut, ByteOrder, BytesMut};

#[derive(Debug, Clone)]
pub struct ResumeOK {
  position: u64,
}

pub struct ResumeOKBuilder {
  stream_id: u32,
  flag: u16,
  value: ResumeOK,
}

impl ResumeOKBuilder {
  fn new(stream_id: u32, flag: u16) -> ResumeOKBuilder {
    ResumeOKBuilder {
      stream_id: stream_id,
      flag: flag,
      value: ResumeOK { position: 0 },
    }
  }
  pub fn set_position(&mut self, position: u64) -> &mut ResumeOKBuilder {
    self.value.position = position;
    self
  }

  pub fn build(&mut self) -> Frame {
    Frame::new(
      self.stream_id,
      Body::ResumeOK(self.value.clone()),
      self.flag,
    )
  }
}

impl ResumeOK {
  pub fn decode(flag: u16, bf: &mut BytesMut) -> RSocketResult<ResumeOK> {
    let p = BigEndian::read_u64(bf);
    bf.advance(4);
    Ok(ResumeOK { position: p })
  }

  pub fn builder(stream_id: u32, flag: u16) -> ResumeOKBuilder {
    ResumeOKBuilder::new(stream_id, flag)
  }

  pub fn get_position(&self) -> u64 {
    self.position
  }
}

impl Writeable for ResumeOK {
  fn write_to(&self, bf: &mut BytesMut) {
    bf.put_u64_be(self.position)
  }

  fn len(&self) -> u32 {
    8
  }
}
