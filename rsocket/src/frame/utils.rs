use super::FLAG_METADATA;
use crate::utils::U24;
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
        let m: Option<Bytes> = if flag & FLAG_METADATA != 0 {
            let n = U24::read_advance(bf);
            Some(bf.split_to(n as usize).freeze())
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
            let n = v.len() as u32;
            U24::write(n, bf);
            bf.put(v.clone());
        }
        if let Some(v) = data {
            bf.put(v.clone())
        }
    }
}
