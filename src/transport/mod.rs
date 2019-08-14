extern crate bytes;
extern crate tokio;

use crate::frame::{Frame, Writeable, U24};
use bytes::{BufMut, BytesMut};
use std::io;
use tokio::codec::{Decoder, Encoder};

pub struct FrameCodec;

impl Decoder for FrameCodec {
  type Item = Frame;
  type Error = io::Error;

  fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
    let l = U24::read(buf);
    let mut bb = buf.split_to(l as usize);
    Ok(Frame::decode(&mut bb))
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
