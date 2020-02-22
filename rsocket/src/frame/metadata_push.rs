use super::{Body, Frame};
use crate::utils::{RSocketResult, Writeable};
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug, PartialEq)]
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
    pub fn decode(flag: u16, bf: &mut BytesMut) -> RSocketResult<MetadataPush> {
        let m = Bytes::from(bf.to_vec());
        Ok(MetadataPush { metadata: Some(m) })
    }

    pub fn builder(stream_id: u32, flag: u16) -> MetadataPushBuiler {
        MetadataPushBuiler::new(stream_id, flag)
    }

    pub fn get_metadata(&self) -> &Option<Bytes> {
        &self.metadata
    }

    pub fn split(self) -> (Option<Bytes>, Option<Bytes>) {
        (None, self.metadata)
    }
}

impl Writeable for MetadataPush {
    fn write_to(&self, bf: &mut BytesMut) {
        match &self.metadata {
            Some(v) => bf.put(v.bytes()),
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
