use super::FLAG_METADATA;
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub struct U24 {}

impl U24 {
  pub fn write(n: u32, bf: &mut BytesMut) {
    bf.put_u8((0xFF & (n >> 16)) as u8);
    bf.put_u8((0xFF & (n >> 8)) as u8);
    bf.put_u8((0xFF & n) as u8);
  }

  pub fn read(bf: &mut BytesMut) -> u32 {
    let mut n: u32 = 0;
    n += u32::from(bf[0]) << 16;
    n += u32::from(bf[1]) << 8;
    n += u32::from(bf[2]);
    n
  }

  pub fn read_advance(bf: &mut BytesMut) -> u32 {
    let mut n: u32 = 0;
    let raw = bf.split_to(3);
    n += u32::from(raw[0]) << 16;
    n += u32::from(raw[1]) << 8;
    n += u32::from(raw[2]);
    n
  }
}

pub struct PayloadSupport {}

impl PayloadSupport {
  pub fn len(metadata: &Option<Bytes>, data: &Option<Bytes>) -> usize {
    let a = match metadata {
      Some(v) => 3 + v.len(),
      None => 0,
    };
    let b = match data {
      Some(v) => v.len(),
      None => 0,
    };
    a + b
  }

  // TODO: change to Result
  pub fn read(flag: u16, bf: &mut BytesMut) -> (Option<Bytes>, Option<Bytes>) {
    let m: Option<Bytes> = if flag & FLAG_METADATA != 0 {
      let n = U24::read_advance(bf);
      Some(bf.split_to(n as usize).to_bytes())
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
    if let Some(v) = metadata {
      let n = v.len() as u32;
      U24::write(n, bf);
      bf.put(v.bytes());
    }
    if let Some(v) = data {
      bf.put(v.bytes())
    }
  }
}
