use super::utils::{read_payload, too_short};
use super::{Body, Frame, PayloadSupport};
use crate::utils::{RSocketResult, Writeable};
use bytes::{BufMut, Bytes, BytesMut};

#[derive(Debug, PartialEq)]
pub struct RequestFNF {
    metadata: Option<Bytes>,
    data: Option<Bytes>,
}

pub struct RequestFNFBuilder {
    stream_id: u32,
    flag: u16,
    value: RequestFNF,
}

impl RequestFNFBuilder {
    fn new(stream_id: u32, flag: u16) -> RequestFNFBuilder {
        RequestFNFBuilder {
            stream_id,
            flag,
            value: RequestFNF {
                metadata: None,
                data: None,
            },
        }
    }

    pub fn build(self) -> Frame {
        Frame::new(self.stream_id, Body::RequestFNF(self.value), self.flag)
    }

    pub fn set_metadata(mut self, metadata: Bytes) -> Self {
        self.value.metadata = Some(metadata);
        self.flag |= Frame::FLAG_METADATA;
        self
    }

    pub fn set_data(mut self, data: Bytes) -> Self {
        self.value.data = Some(data);
        self
    }

    pub fn set_all(mut self, data_and_metadata: (Option<Bytes>, Option<Bytes>)) -> Self {
        self.value.data = data_and_metadata.0;
        match data_and_metadata.1 {
            Some(m) => {
                self.value.metadata = Some(m);
                self.flag |= Frame::FLAG_METADATA;
            }
            None => {
                self.value.metadata = None;
                self.flag &= !Frame::FLAG_METADATA;
            }
        }
        self
    }
}

impl RequestFNF {
    pub(crate) fn decode(flag: u16, bf: &mut BytesMut) -> RSocketResult<RequestFNF> {
        read_payload(flag, bf).map(|(m, d)| RequestFNF {
            metadata: m,
            data: d,
        })
    }

    pub fn builder(stream_id: u32, flag: u16) -> RequestFNFBuilder {
        RequestFNFBuilder::new(stream_id, flag)
    }

    pub fn get_metadata(&self) -> Option<&Bytes> {
        match &self.metadata {
            Some(b) => Some(b),
            None => None,
        }
    }

    pub fn get_data(&self) -> Option<&Bytes> {
        match &self.data {
            Some(b) => Some(b),
            None => None,
        }
    }

    pub fn split(self) -> (Option<Bytes>, Option<Bytes>) {
        (self.data, self.metadata)
    }
}

impl Writeable for RequestFNF {
    fn write_to(&self, bf: &mut BytesMut) {
        PayloadSupport::write(bf, self.get_metadata(), self.get_data());
    }

    fn len(&self) -> usize {
        PayloadSupport::len(self.get_metadata(), self.get_data())
    }
}
