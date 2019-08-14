extern crate bytes;
use bytes::{BufMut, Bytes, BytesMut};

pub struct U24 {}

impl U24 {
  pub fn write(n: u32, bf: &mut BytesMut) {
    bf.put_u8((0xFF & (n >> 16)) as u8);
    bf.put_u8((0xFF & (n >> 8)) as u8);
    bf.put_u8((0xFF & n) as u8);
  }

  pub fn read(bf: &mut BytesMut) -> u32 {
    let mut n: u32 = 0;
    let bar = bf.split_to(3);
    n += (bar[0] as u32) << 16;
    n += (bar[1] as u32) << 8;
    n += bar[2] as u32;
    n
  }
}
