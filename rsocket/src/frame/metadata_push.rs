use bytes::{Buf, BufMut, Bytes, BytesMut};

use super::{Body, Frame};
use crate::utils::Writeable;

#[derive(Debug, Eq, PartialEq)]
pub struct MetadataPush {
    metadata: Option<Bytes>,
}

pub struct MetadataPushBuiler {
    stream_id: u32,
    flag: u16,
    value: MetadataPush,
}

impl MetadataPushBuiler {
    fn new(stream_id: u32, flag: u16) -> MetadataPushBuiler {
        MetadataPushBuiler {
            stream_id,
            flag,
            value: MetadataPush { metadata: None },
        }
    }

    pub fn set_metadata(mut self, metadata: Bytes) -> Self {
        self.value.metadata = Some(metadata);
        self
    }

    pub fn build(self) -> Frame {
        Frame::new(self.stream_id, Body::MetadataPush(self.value), self.flag)
    }
}

impl MetadataPush {
    pub(crate) fn decode(flag: u16, bf: &mut BytesMut) -> crate::Result<MetadataPush> {
        Ok(MetadataPush {
            metadata: Some(bf.split().freeze()),
        })
    }

    pub fn builder(stream_id: u32, flag: u16) -> MetadataPushBuiler {
        MetadataPushBuiler::new(stream_id, flag)
    }

    pub fn get_metadata(&self) -> Option<&Bytes> {
        self.metadata.as_ref()
    }

    pub fn split(self) -> (Option<Bytes>, Option<Bytes>) {
        (None, self.metadata)
    }
}

impl Writeable for MetadataPush {
    fn write_to(&self, bf: &mut BytesMut) {
        match &self.metadata {
            Some(v) => bf.extend_from_slice(v),
            None => (),
        }
    }

    fn len(&self) -> usize {
        match &self.metadata {
            Some(v) => v.len(),
            None => 0,
        }
    }
}
