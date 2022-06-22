use bytes::{BufMut, Bytes, BytesMut};

use super::{utils, Body, Frame};
use crate::utils::Writeable;

#[derive(Debug, Eq, PartialEq)]
pub struct RequestResponse {
    metadata: Option<Bytes>,
    data: Option<Bytes>,
}

pub struct RequestResponseBuilder {
    stream_id: u32,
    flag: u16,
    value: RequestResponse,
}

impl RequestResponseBuilder {
    fn new(stream_id: u32, flag: u16) -> RequestResponseBuilder {
        RequestResponseBuilder {
            stream_id,
            flag,
            value: RequestResponse {
                metadata: None,
                data: None,
            },
        }
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

    pub fn set_metadata(mut self, metadata: Bytes) -> Self {
        self.value.metadata = Some(metadata);
        self.flag |= Frame::FLAG_METADATA;
        self
    }

    pub fn set_data(mut self, data: Bytes) -> Self {
        self.value.data = Some(data);
        self
    }

    pub fn build(self) -> Frame {
        Frame::new(self.stream_id, Body::RequestResponse(self.value), self.flag)
    }
}

impl RequestResponse {
    pub(crate) fn decode(flag: u16, bf: &mut BytesMut) -> crate::Result<RequestResponse> {
        utils::read_payload(flag, bf).map(|(m, d)| RequestResponse {
            metadata: m,
            data: d,
        })
    }

    pub fn builder(stream_id: u32, flag: u16) -> RequestResponseBuilder {
        RequestResponseBuilder::new(stream_id, flag)
    }

    pub fn get_metadata(&self) -> Option<&Bytes> {
        self.metadata.as_ref()
    }

    pub fn get_data(&self) -> Option<&Bytes> {
        self.data.as_ref()
    }

    pub fn split(self) -> (Option<Bytes>, Option<Bytes>) {
        (self.data, self.metadata)
    }
}

impl Writeable for RequestResponse {
    fn write_to(&self, bf: &mut BytesMut) {
        utils::write_payload(bf, self.get_metadata(), self.get_data())
    }

    fn len(&self) -> usize {
        utils::calculate_payload_length(self.get_metadata(), self.get_data())
    }
}
