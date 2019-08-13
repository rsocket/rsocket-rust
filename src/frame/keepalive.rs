extern crate bytes;

use crate::frame::{Body, Frame};
use bytes::{BigEndian, BufMut, Bytes, BytesMut};

#[derive(Debug, Clone)]
pub struct Keepalive {
  last_received_position: i64,
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

  pub fn set_last_received_position(&mut self, position: i64) -> &mut KeepaliveBuilder {
    if position < 0 {
      self.keepalive.last_received_position = 0
    } else {
      self.keepalive.last_received_position = position;
    }
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

impl Keepalive {
  pub fn builder(stream_id: u32, flag: u16) -> KeepaliveBuilder {
    KeepaliveBuilder::new(stream_id, flag)
  }

  pub fn get_last_received_position(&self) -> i64 {
    self.last_received_position.clone()
  }

  pub fn get_data(&self) -> Option<Bytes> {
    self.data.clone()
  }
}
