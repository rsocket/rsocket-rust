use super::mime::MimeType;
use crate::error::RSocketError;
use crate::utils::{u24, Writeable};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::collections::LinkedList;
use std::convert::TryFrom;
use std::result::Result;

const MAX_MIME_LEN: usize = 0x7F + 1;

#[derive(Debug, Clone, Eq, PartialEq, Default)]
pub struct CompositeMetadata {
    metadatas: LinkedList<CompositeMetadataEntry>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CompositeMetadataEntry {
    mime_type: MimeType,
    metadata: Bytes,
}

pub struct CompositeMetadataBuilder {
    inner: CompositeMetadata,
}

impl CompositeMetadataBuilder {
    pub fn push<A>(mut self, mime_type: MimeType, payload: A) -> Self
    where
        A: AsRef<[u8]>,
    {
        let mut bf = BytesMut::new();
        bf.put_slice(payload.as_ref());
        let m = CompositeMetadataEntry::new(mime_type, bf.freeze());
        self.inner.push(m);
        self
    }

    pub fn push_entry(mut self, metadata: CompositeMetadataEntry) -> Self {
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

    pub fn decode(b: &mut BytesMut) -> crate::Result<CompositeMetadata> {
        let mut metadatas = LinkedList::new();
        loop {
            match Self::decode_once(b) {
                Ok(Some(v)) => metadatas.push_back(v),
                Ok(None) => break,
                Err(e) => return Err(e),
            }
        }
        Ok(CompositeMetadata { metadatas })
    }

    pub fn iter(&self) -> impl Iterator<Item = &CompositeMetadataEntry> {
        self.metadatas.iter()
    }

    #[inline]
    fn decode_once(bs: &mut BytesMut) -> crate::Result<Option<CompositeMetadataEntry>> {
        if bs.is_empty() {
            return Ok(None);
        }
        let first: u8 = bs.get_u8();
        let mime_type = if 0x80 & first != 0 {
            // Well
            let n = first & 0x7F;
            match MimeType::parse(n) {
                Some(well) => well,
                None => {
                    let err_str = format!("invalid Well-Known MIME type: identifier={:x}", n);
                    return Err(RSocketError::WithDescription(err_str).into());
                }
            }
        } else {
            // Bad
            let mime_len = (first as usize) + 1;
            if bs.len() < mime_len {
                return Err(RSocketError::WithDescription(
                    "broken composite metadata: empty MIME type!".into(),
                )
                .into());
            }
            let front = bs.split_to(mime_len);
            MimeType::Normal(String::from_utf8(front.to_vec()).unwrap())
        };

        if bs.len() < 3 {
            return Err(RSocketError::WithDescription(
                "broken composite metadata: not enough bytes!".into(),
            )
            .into());
        }
        let payload_size = u24::read_advance(bs).into();
        if bs.len() < payload_size {
            let desc = format!("broken composite metadata: require {} bytes!", payload_size);
            return Err(RSocketError::WithDescription(desc).into());
        }
        let metadata = bs.split_to(payload_size).freeze();
        Ok(Some(CompositeMetadataEntry::new(mime_type, metadata)))
    }

    pub fn push(&mut self, metadata: CompositeMetadataEntry) {
        self.metadatas.push_back(metadata)
    }
}

impl CompositeMetadataEntry {
    pub fn new(mime_type: MimeType, metadata: Bytes) -> CompositeMetadataEntry {
        assert!(metadata.len() <= (u24::MAX as usize));
        CompositeMetadataEntry {
            mime_type,
            metadata,
        }
    }

    pub fn get_mime_type(&self) -> &MimeType {
        &self.mime_type
    }

    pub fn get_metadata(&self) -> &Bytes {
        &self.metadata
    }

    pub fn get_metadata_utf8(&self) -> &str {
        std::str::from_utf8(self.metadata.as_ref()).expect("Invalid UTF-8 bytes.")
    }
}

impl Writeable for CompositeMetadataEntry {
    fn write_to(&self, bf: &mut BytesMut) {
        match &self.mime_type {
            MimeType::WellKnown(n) => {
                // WellKnown
                bf.put_u8(0x80 | n);
            }
            MimeType::Normal(s) => {
                // NotWellKnown
                let mime_type_len = s.len() as u8;
                assert!(mime_type_len > 0, "invalid length of MimeType!");
                bf.put_u8(mime_type_len - 1);
                bf.put_slice(s.as_ref());
            }
        };
        let metadata_len = self.metadata.len();
        u24::from(metadata_len).write_to(bf);
        if metadata_len > 0 {
            bf.put(self.metadata.bytes());
        }
    }

    fn len(&self) -> usize {
        // 1byte(MIME) + 3bytes(length of payload in u24)
        let mut amount = 4;
        if let MimeType::Normal(s) = &self.mime_type {
            amount += s.len();
        }
        amount += self.metadata.len();
        amount
    }
}
