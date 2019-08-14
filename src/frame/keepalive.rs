extern crate bytes;

use crate::frame::{Body, Frame, Writeable};
use bytes::{BigEndian, BufMut, ByteOrder, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct Keepalive {
  last_received_position: u64,
  data: Option<Bytes>,
}

pub struct KeepaliveBuilder {
  stream_id: u32,
  flag: u16,
  keepalive: Keepalive,
}

impl KeepaliveBuilder {
  fn new(stream_id: u32, flag: u16) -> KeepaliveBuilder {
    KeepaliveBuilder {
      stream_id: stream_id,
      flag: flag,
      keepalive: Keepalive {
        last_received_position: 0,
        data: None,
      },
    }
  }

  pub fn set_data(&mut self, data: Bytes) -> &mut KeepaliveBuilder {
    self.keepalive.data = Some(data);
    self
  }

  pub fn set_last_received_position(&mut self, position: u64) -> &mut KeepaliveBuilder {
    self.keepalive.last_received_position = position;
    self
  }

  pub fn build(&mut self) -> Frame {
    Frame::new(
      self.stream_id,
      Body::Keepalive(self.keepalive.clone()),
      self.flag,
    )
  }
}

impl Writeable for Keepalive {
  fn write_to(&self, bf: &mut BytesMut) {
    bf.put_u64_be(self.last_received_position);
    match &self.data {
      Some(v) => bf.put(v),
      None => (),
    }
  }

  fn len(&self) -> u32 {
    match &self.data {
      Some(v) => 8 + (v.len() as u32),
      None => 8,
    }
  }
}

impl Keepalive {
  pub fn decode(flag: u16, bf: &mut Bytes) -> Option<Keepalive> {
    let position = BigEndian::read_u64(bf);
    bf.advance(8);
    let mut d: Option<Bytes> = None;
    if !bf.is_empty() {
      d = Some(Bytes::from(bf.to_vec()));
    }
    Some(Keepalive {
      last_received_position: position,
      data: d,
    })
  }

  pub fn builder(stream_id: u32, flag: u16) -> KeepaliveBuilder {
    KeepaliveBuilder::new(stream_id, flag)
  }

  pub fn get_last_received_position(&self) -> u64 {
    self.last_received_position.clone()
  }

  pub fn get_data(&self) -> Option<Bytes> {
    self.data.clone()
  }
}
