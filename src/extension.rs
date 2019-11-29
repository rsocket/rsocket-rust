use super::mime::WellKnownMIME;
use crate::errors::{ErrorKind, RSocketError};
use crate::frame::U24;
use crate::result::RSocketResult;
use bytes::{Buf, BufMut, Bytes, BytesMut};

const MAX_MIME_LEN: usize = 0x7F;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompositeMetadata {
    mime: String,
    payload: Bytes,
}

impl CompositeMetadata {
    pub fn new(mime: String, payload: Bytes) -> CompositeMetadata {
        if mime.len() > MAX_MIME_LEN {
            panic!("too large MIME type!");
        }
        if payload.len() > U24::max() {
            panic!("too large Payload!")
        }
        CompositeMetadata { mime, payload }
    }

    pub fn decode(b: &mut BytesMut) -> RSocketResult<Vec<CompositeMetadata>> {
        let mut metadatas: Vec<CompositeMetadata> = vec![];
        loop {
            match Self::decode_once(b) {
                Ok(op) => match op {
                    Some(v) => metadatas.push(v),
                    None => break,
                },
                Err(e) => return Err(e),
            }
        }
        Ok(metadatas)
    }

    fn decode_once(bs: &mut BytesMut) -> RSocketResult<Option<CompositeMetadata>> {
        if bs.is_empty() {
            return Ok(None);
        }
        let first: u8 = bs.get_u8();
        let m = if 0x80 & first != 0 {
            // Well
            let well = WellKnownMIME::from(first & 0x7F);
            well.str().to_string()
        } else {
            // Bad
            let mime_len = first as usize;
            if bs.len() < mime_len {
                return Err(RSocketError::from(ErrorKind::WithDescription(
                    "bad COMPOSITE_METADATA bytes: missing required bytes!",
                )));
            }
            let front = bs.split_to(mime_len);
            String::from_utf8(front.to_vec()).unwrap()
        };

        if bs.len() < 3 {
            return Err(RSocketError::from(ErrorKind::WithDescription(
                "bad COMPOSITE_METADATA bytes: missing required bytes!",
            )));
        }
        let payload_size = U24::read_advance(bs) as usize;
        if bs.len() < payload_size {
            return Err(RSocketError::from(ErrorKind::WithDescription(
                "bad COMPOSITE_METADATA bytes: missing required bytes!",
            )));
        }
        let p = bs.split_to(payload_size).freeze();
        Ok(Some(CompositeMetadata::new(m, p)))
    }

    pub fn get_mime(&self) -> &String {
        &self.mime
    }

    pub fn get_payload(&self) -> &Bytes {
        &self.payload
    }

    pub fn write_to(&self, bf: &mut BytesMut) {
        let mi = WellKnownMIME::from(self.mime.as_str());
        let first_byte: u8 = if mi == WellKnownMIME::Unknown {
            // Bad
            self.mime.len() as u8
        } else {
            // Goodmi
            0x80 | mi.raw()
        };
        let payload_size = self.payload.len();

        bf.put_u8(first_byte);
        if first_byte & 0x80 == 0 {
            bf.put_slice(self.mime.as_bytes());
        }
        U24::write(payload_size as u32, bf);
        bf.put(self.payload.bytes());
    }

    pub fn bytes(&self) -> Bytes {
        let mut bf = BytesMut::new();
        self.write_to(&mut bf);
        bf.freeze()
    }
}
