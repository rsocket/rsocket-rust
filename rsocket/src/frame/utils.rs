use super::Frame;
use crate::utils::{u24, Writeable};
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub(crate) struct PayloadSupport {}

impl PayloadSupport {
    pub fn len(metadata: Option<&Bytes>, data: Option<&Bytes>) -> usize {
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

    pub fn read(flag: u16, bf: &mut BytesMut) -> (Option<Bytes>, Option<Bytes>) {
        let m: Option<Bytes> = if flag & Frame::FLAG_METADATA != 0 {
            let n = u24::read_advance(bf);
            Some(bf.split_to(n.into()).freeze())
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

    pub fn write(bf: &mut BytesMut, metadata: Option<&Bytes>, data: Option<&Bytes>) {
        if let Some(v) = metadata {
            u24::from(v.len()).write_to(bf);
            bf.put(v.clone());
        }
        if let Some(v) = data {
            bf.put(v.clone())
        }
    }
}
