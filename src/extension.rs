use super::mime::WellKnownMIME;
use crate::errors::{ErrorKind, RSocketError};
use crate::frame::U24;
use crate::result::RSocketResult;
use bytes::{Buf, BufMut, Bytes, BytesMut};

const MAX_MIME_LEN: usize = 0x7F;
const MAX_ROUTING_TAG_LEN: usize = 0xFF;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompositeMetadata {
    mime: String,
    payload: Bytes,
}

#[derive(Debug, Clone)]
pub struct RoutingMetadata {
    tags: Vec<String>,
}

pub struct RoutingMetadataBuilder {
    inner: RoutingMetadata,
}

impl RoutingMetadataBuilder {
    pub fn push_str(self, tag: &str) -> Self {
        self.push(String::from(tag))
    }
    pub fn push(mut self, tag: String) -> Self {
        if tag.len() > MAX_ROUTING_TAG_LEN {
            panic!("exceeded maximum routing tag length!");
        }
        self.inner.tags.push(tag);
        self
    }
    pub fn build(self) -> RoutingMetadata {
        self.inner
    }
}

impl RoutingMetadata {
    pub fn builder() -> RoutingMetadataBuilder {
        RoutingMetadataBuilder {
            inner: RoutingMetadata { tags: vec![] },
        }
    }

    pub fn decode(bf: &mut BytesMut) -> RSocketResult<RoutingMetadata> {
        let mut bu = RoutingMetadata::builder();
        loop {
            match Self::decode_once(bf) {
                Ok(v) => match v {
                    Some(tag) => bu = bu.push(tag),
                    None => break,
                },
                Err(e) => return Err(e),
            }
        }
        Ok(bu.build())
    }

    fn decode_once(bf: &mut BytesMut) -> RSocketResult<Option<String>> {
        if bf.is_empty() {
            return Ok(None);
        }
        let size = bf.get_u8() as usize;
        if bf.len() < size {
            return Err(RSocketError::from("require more bytes!"));
        }
        let tag = String::from_utf8(bf.split_to(size).to_vec()).unwrap();
        Ok(Some(tag))
    }

    pub fn get_tags(&self) -> &Vec<String> {
        &self.tags
    }

    pub fn write_to(&self, bf: &mut BytesMut) {
        for tag in &self.tags {
            let size = tag.len() as u8;
            bf.put_u8(size);
            bf.put_slice(tag.as_bytes());
        }
    }

    pub fn bytes(&self) -> Bytes {
        let mut bf = BytesMut::new();
        self.write_to(&mut bf);
        bf.freeze()
    }
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
                return Err(RSocketError::from("broken COMPOSITE_METADATA bytes!"));
            }
            let front = bs.split_to(mime_len);
            String::from_utf8(front.to_vec()).unwrap()
        };

        if bs.len() < 3 {
            return Err(RSocketError::from("broken COMPOSITE_METADATA bytes!"));
        }
        let payload_size = U24::read_advance(bs) as usize;
        if bs.len() < payload_size {
            return Err(RSocketError::from("broken COMPOSITE_METADATA bytes!"));
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
