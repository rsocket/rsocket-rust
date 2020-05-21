use super::Frame;
use crate::error::{ErrorKind, RSocketError};
use crate::utils::{u24, RSocketResult, Writeable};
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub(crate) fn read_payload(
    flag: u16,
    bf: &mut BytesMut,
) -> RSocketResult<(Option<Bytes>, Option<Bytes>)> {
    let m: Option<Bytes> = if flag & Frame::FLAG_METADATA != 0 {
        if bf.len() < 3 {
            return too_short(3);
        }
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
    Ok((m, d))
}

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

pub(crate) fn too_short<T>(n: usize) -> RSocketResult<T> {
    Err(ErrorKind::LengthTooShort(n).into())
}
