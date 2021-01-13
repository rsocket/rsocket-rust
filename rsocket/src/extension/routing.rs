use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::error::RSocketError;
use crate::utils::Writeable;

const MAX_ROUTING_TAG_LEN: usize = 0xFF;

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
        assert!(
            tag.len() <= MAX_ROUTING_TAG_LEN,
            "exceeded maximum routing tag length!"
        );
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

    pub fn decode(bf: &mut BytesMut) -> crate::Result<RoutingMetadata> {
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

    pub fn get_tags(&self) -> &Vec<String> {
        &self.tags
    }

    fn decode_once(bf: &mut BytesMut) -> crate::Result<Option<String>> {
        if bf.is_empty() {
            return Ok(None);
        }
        let size = bf.get_u8() as usize;
        if bf.len() < size {
            return Err(RSocketError::WithDescription("require more bytes!".into()).into());
        }
        let tag = String::from_utf8(bf.split_to(size).to_vec())?;
        Ok(Some(tag))
    }
}

impl Writeable for RoutingMetadata {
    fn write_to(&self, bf: &mut BytesMut) {
        for tag in &self.tags {
            let size = tag.len() as u8;
            bf.put_u8(size);
            bf.put_slice(tag.as_bytes());
        }
    }

    fn len(&self) -> usize {
        let mut n = 0;
        for tag in &self.tags {
            n += 1 + tag.as_bytes().len();
        }
        n
    }
}
