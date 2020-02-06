use crate::errors::{ErrorKind, RSocketError};
use crate::mime::WellKnownMIME;
use crate::utils::{RSocketResult, Writeable, U24};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::collections::LinkedList;
use std::result::Result;

const MAX_MIME_LEN: usize = 0x7F;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct CompositeMetadata {
    metadatas: LinkedList<Metadata>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Metadata {
    mime: String,
    payload: Bytes,
}

pub struct CompositeMetadataBuilder {
    inner: CompositeMetadata,
}

impl CompositeMetadataBuilder {
    pub fn push<I, A>(mut self, mime: I, payload: A) -> Self
    where
        I: Into<String>,
        A: AsRef<[u8]>,
    {
        let mut bf = BytesMut::new();
        bf.put_slice(payload.as_ref());
        let m = Metadata::new(mime.into(), bf.freeze());
        self.inner.push(m);
        self
    }

    pub fn push_metadata(mut self, metadata: Metadata) -> Self {
        self.inner.push(metadata);
        self
    }

    pub fn build(self) -> CompositeMetadata {
        self.inner
    }
}

impl Into<Vec<u8>> for CompositeMetadata {
    fn into(self) -> Vec<u8> {
        let mut bf = BytesMut::new();
        self.write_to(&mut bf);
        bf.to_vec()
    }
}

impl Into<Bytes> for CompositeMetadata {
    fn into(self) -> Bytes {
        let mut bf = BytesMut::new();
        self.write_to(&mut bf);
        bf.freeze()
    }
}

impl Into<BytesMut> for CompositeMetadata {
    fn into(self) -> BytesMut {
        let mut bf = BytesMut::new();
        self.write_to(&mut bf);
        bf
    }
}

impl Writeable for CompositeMetadata {
    fn write_to(&self, bf: &mut BytesMut) {
        for it in self.iter() {
            it.write_to(bf);
        }
    }

    fn len(&self) -> usize {
        let mut n = 0;
        for it in self.iter() {
            n += it.len();
        }
        n
    }
}

impl CompositeMetadata {
    pub fn builder() -> CompositeMetadataBuilder {
        CompositeMetadataBuilder {
            inner: CompositeMetadata::default(),
        }
    }

    pub fn decode(b: &mut BytesMut) -> RSocketResult<CompositeMetadata> {
        let mut metadatas = LinkedList::new();
        loop {
            match Self::decode_once(b) {
                Ok(op) => match op {
                    Some(v) => metadatas.push_back(v),
                    None => break,
                },
                Err(e) => return Err(e),
            }
        }
        Ok(CompositeMetadata { metadatas })
    }

    pub fn iter(&self) -> impl Iterator<Item = &Metadata> {
        self.metadatas.iter()
    }

    #[inline]
    fn decode_once(bs: &mut BytesMut) -> RSocketResult<Option<Metadata>> {
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
        Ok(Some(Metadata::new(m, p)))
    }

    pub fn push(&mut self, metadata: Metadata) {
        self.metadatas.push_back(metadata)
    }
}

impl Metadata {
    pub fn new(mime: String, payload: Bytes) -> Metadata {
        if mime.len() > MAX_MIME_LEN {
            panic!("too large MIME type!");
        }
        if payload.len() > U24::max() {
            panic!("too large Payload!")
        }
        Metadata { mime, payload }
    }

    pub fn get_mime(&self) -> &String {
        &self.mime
    }

    pub fn get_payload(&self) -> &Bytes {
        &self.payload
    }
}

impl Writeable for Metadata {
    fn write_to(&self, bf: &mut BytesMut) {
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

    fn len(&self) -> usize {
        let mut amount = 4;
        let wellknown = WellKnownMIME::from(self.mime.as_str()) != WellKnownMIME::Unknown;
        if wellknown {
            amount += self.mime.len();
        }
        amount += self.payload.len();
        amount
    }
}
