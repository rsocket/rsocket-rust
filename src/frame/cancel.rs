use crate::frame::{Body, Frame};

#[derive(Debug, Clone)]
pub struct Cancel {}

impl Cancel {
  pub fn new(stream_id: u32, flag: u16) -> Frame {
    Frame::new(stream_id, Body::Cancel(), flag)
  }
}
