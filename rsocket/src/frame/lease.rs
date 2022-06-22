use bytes::{Buf, BufMut, Bytes, BytesMut};

use super::{Body, Frame};
use crate::error::RSocketError;
use crate::utils::Writeable;

#[derive(Debug, Eq, PartialEq)]
pub struct Lease {
    ttl: u32,
    number_of_requests: u32,
    metadata: Option<Bytes>,
}

pub struct LeaseBuilder {
    stream_id: u32,
    flag: u16,
    value: Lease,
}

impl LeaseBuilder {
    fn new(stream_id: u32, flag: u16) -> LeaseBuilder {
        LeaseBuilder {
            stream_id,
            flag,
            value: Lease {
                ttl: 0,
                number_of_requests: 0,
                metadata: None,
            },
        }
    }

    pub fn set_metadata(mut self, metadata: Bytes) -> Self {
        self.value.metadata = Some(metadata);
        self.flag |= Frame::FLAG_METADATA;
        self
    }

    pub fn set_ttl(mut self, ttl: u32) -> Self {
        self.value.ttl = ttl;
        self
    }

    pub fn set_number_of_requests(mut self, n: u32) -> Self {
        self.value.number_of_requests = n;
        self
    }

    pub fn build(self) -> Frame {
        Frame::new(self.stream_id, Body::Lease(self.value), self.flag)
    }
}

impl Lease {
    pub(crate) fn decode(flag: u16, bf: &mut BytesMut) -> crate::Result<Lease> {
        if bf.len() < 8 {
            return Err(RSocketError::InCompleteFrame.into());
        }
        let ttl = bf.get_u32();
        let n = bf.get_u32();
        let m = if flag & Frame::FLAG_METADATA != 0 {
            Some(bf.split().freeze())
        } else {
            None
        };
        Ok(Lease {
            ttl,
            number_of_requests: n,
            metadata: m,
        })
    }

    pub fn builder(stream_id: u32, flag: u16) -> LeaseBuilder {
        LeaseBuilder::new(stream_id, flag)
    }

    pub fn get_number_of_requests(&self) -> u32 {
        self.number_of_requests
    }

    pub fn get_metadata(&self) -> Option<&Bytes> {
        self.metadata.as_ref()
    }

    pub fn get_ttl(&self) -> u32 {
        self.ttl
    }
}

impl Writeable for Lease {
    fn write_to(&self, bf: &mut BytesMut) {
        bf.put_u32(self.ttl);
        bf.put_u32(self.number_of_requests);
        match &self.metadata {
            Some(v) => bf.extend_from_slice(v),
            None => (),
        }
    }

    fn len(&self) -> usize {
        8 + match &self.metadata {
            Some(v) => v.len(),
            None => 0,
        }
    }
}
