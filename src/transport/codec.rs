extern crate bytes;
extern crate tokio;

use crate::errors::RSocketError;
use crate::frame::{Frame, Writeable, U24};
use bytes::{BufMut, BytesMut};
use std::io;
use tokio::codec::{Decoder, Encoder};

pub struct FrameCodec;

impl Decoder for FrameCodec {
  type Item = Frame;
  type Error = RSocketError;

  fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    let actual = buf.len();
    if actual < 3 {
      return Ok(None);
    }
    let l = U24::read(buf) as usize;
    if actual < 3 + l {
      return Ok(None);
    }
    let mut bb = buf.split_to(l);
    Frame::decode(&mut bb).map(|it| Some(it))
  }
}

impl Encoder for FrameCodec {
  type Item = Frame;
  type Error = io::Error;
  fn encode(&mut self, item: Frame, buf: &mut BytesMut) -> Result<(), Self::Error> {
    let l = item.len();
    U24::write(l, buf);
    item.write_to(buf);
    Ok(())
  }
}

impl FrameCodec {
  pub fn new() -> FrameCodec {
    FrameCodec {}
  }
}
