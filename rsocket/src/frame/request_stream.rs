use bytes::{Buf, BufMut, Bytes, BytesMut};

use super::{utils, Body, Frame, REQUEST_MAX};
use crate::error::RSocketError;
use crate::utils::Writeable;

#[derive(Debug, Eq, PartialEq)]
pub struct RequestStream {
    initial_request_n: u32,
    metadata: Option<Bytes>,
    data: Option<Bytes>,
}

pub struct RequestStreamBuilder {
    stream_id: u32,
    flag: u16,
    value: RequestStream,
}

impl RequestStreamBuilder {
    pub fn build(self) -> Frame {
        Frame::new(self.stream_id, Body::RequestStream(self.value), self.flag)
    }

    pub fn set_initial_request_n(mut self, n: u32) -> Self {
        self.value.initial_request_n = n;
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

    pub fn set_data(mut self, data: Bytes) -> Self {
        self.value.data = Some(data);
        self
    }

    pub fn set_metadata(mut self, metadata: Bytes) -> Self {
        self.value.metadata = Some(metadata);
        self.flag |= Frame::FLAG_METADATA;
        self
    }
}

impl RequestStream {
    pub(crate) fn decode(flag: u16, bf: &mut BytesMut) -> crate::Result<RequestStream> {
        if bf.len() < 4 {
            Err(RSocketError::InCompleteFrame.into())
        } else {
            let initial_request_n = bf.get_u32();
            utils::read_payload(flag, bf).map(move |(metadata, data)| RequestStream {
                initial_request_n,
                metadata,
                data,
            })
        }
    }

    pub fn builder(stream_id: u32, flag: u16) -> RequestStreamBuilder {
        RequestStreamBuilder {
            stream_id,
            flag,
            value: RequestStream {
                initial_request_n: REQUEST_MAX,
                metadata: None,
                data: None,
            },
        }
    }

    pub fn get_initial_request_n(&self) -> u32 {
        self.initial_request_n
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

impl Writeable for RequestStream {
    fn write_to(&self, bf: &mut BytesMut) {
        bf.put_u32(self.initial_request_n);
        utils::write_payload(bf, self.get_metadata(), self.get_data())
    }

    fn len(&self) -> usize {
        4 + utils::calculate_payload_length(self.get_metadata(), self.get_data())
    }
}
