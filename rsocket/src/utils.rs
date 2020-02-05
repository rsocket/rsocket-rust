use crate::errors::RSocketError;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::result::Result;

pub const DEFAULT_MIME_TYPE: &str = "application/binary";

pub type RSocketResult<T> = Result<T, RSocketError>;

pub trait Writeable {
    fn write_to(&self, bf: &mut BytesMut);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
}

pub struct U24;

impl U24 {
    pub fn max() -> usize {
        0x00FF_FFFF
    }

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
