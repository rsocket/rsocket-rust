use super::Frame;
use crate::error::RSocketError;
use crate::utils::{u24, Writeable};
use bytes::{Buf, BufMut, Bytes, BytesMut};

pub(crate) fn read_payload(
    flag: u16,
    bf: &mut BytesMut,
) -> crate::Result<(Option<Bytes>, Option<Bytes>)> {
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

pub(crate) fn calculate_payload_length(metadata: Option<&Bytes>, data: Option<&Bytes>) -> usize {
    metadata.map(|v| 3 + v.len()).unwrap_or(0) + data.map(|v| v.len()).unwrap_or(0)
}

pub(crate) fn write_payload(bf: &mut BytesMut, metadata: Option<&Bytes>, data: Option<&Bytes>) {
    if let Some(v) = metadata {
        u24::from(v.len()).write_to(bf);
        bf.put(v.clone());
    }
    if let Some(v) = data {
        bf.put(v.clone())
    }
}

pub(crate) fn too_short<T>(n: usize) -> crate::Result<T> {
    Err(RSocketError::InCompleteFrame.into())
}
