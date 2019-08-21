extern crate bytes;

use crate::frame::FLAG_METADATA;
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

pub struct PayloadSupport {}

impl PayloadSupport {
  pub fn len(metadata: &Option<Bytes>, data: &Option<Bytes>) -> u32 {
    let a: u32 = match metadata {
      Some(v) => 3 + (v.len() as u32),
      None => 0,
    };
    let b: u32 = match data {
      Some(v) => v.len() as u32,
      None => 0,
    };
    a + b
  }

  // TODO: change to Result
  pub fn read(flag: u16, bf: &mut BytesMut) -> (Option<Bytes>, Option<Bytes>) {
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
    (m, d)
  }

  pub fn write(bf: &mut BytesMut, metadata: &Option<Bytes>, data: &Option<Bytes>) {
    match metadata {
      Some(v) => {
        let n = v.len() as u32;
        U24::write(n, bf);
        bf.put(v);
      }
      None => (),
    }
    match data {
      Some(v) => bf.put(v),
      None => (),
    }
  }
}
