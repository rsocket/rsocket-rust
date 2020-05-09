use bytes::{BufMut, BytesMut};
use rsocket_rust::extension::MimeType;
use serde::{Deserialize, Serialize};
use std::error::Error;

pub(crate) fn unmarshal<'a, T>(mime_type: &MimeType, raw: &'a [u8]) -> Result<T, Box<dyn Error>>
where
    T: Deserialize<'a>,
{
    match *mime_type {
        MimeType::APPLICATION_JSON => Ok(serde_json::from_slice(raw)?),
        MimeType::APPLICATION_CBOR => Ok(serde_cbor::from_slice(raw)?),
        _ => panic!(""),
    }
}

pub(crate) fn marshal<T>(
    mime_type: &MimeType,
    bf: &mut BytesMut,
    data: &T,
) -> Result<(), Box<dyn Error>>
where
    T: Sized + Serialize,
{
    match *mime_type {
        MimeType::APPLICATION_JSON => {
            let raw = serde_json::to_vec(data)?;
            bf.put_slice(&raw[..]);
            Ok(())
        }
        MimeType::APPLICATION_CBOR => {
            let raw = serde_cbor::to_vec(data)?;
            bf.put_slice(&raw[..]);
            Ok(())
        }
        _ => panic!(""),
    }
}
