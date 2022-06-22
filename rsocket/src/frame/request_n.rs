use bytes::{Buf, BufMut, Bytes, BytesMut};

use super::{utils, Body, Frame, REQUEST_MAX};
use crate::error::RSocketError;
use crate::utils::Writeable;

#[derive(Debug, Eq, PartialEq)]
pub struct RequestN {
    n: u32,
}

pub struct RequestNBuilder {
    stream_id: u32,
    flag: u16,
    value: RequestN,
}

impl RequestNBuilder {
    fn new(stream_id: u32, flag: u16) -> RequestNBuilder {
        RequestNBuilder {
            stream_id,
            flag,
            value: RequestN { n: REQUEST_MAX },
        }
    }

    pub fn set_n(mut self, n: u32) -> Self {
        self.value.n = n;
        self
    }

    pub fn build(self) -> Frame {
        Frame::new(self.stream_id, Body::RequestN(self.value), self.flag)
    }
}

impl RequestN {
    pub(crate) fn decode(flag: u16, bf: &mut BytesMut) -> crate::Result<RequestN> {
        if bf.len() < 4 {
            Err(RSocketError::InCompleteFrame.into())
        } else {
            let n = bf.get_u32();
            Ok(RequestN { n })
        }
    }

    pub fn builder(stream_id: u32, flag: u16) -> RequestNBuilder {
        RequestNBuilder::new(stream_id, flag)
    }

    pub fn get_n(&self) -> u32 {
        self.n
    }
}

impl Writeable for RequestN {
    fn write_to(&self, bf: &mut BytesMut) {
        bf.put_u32(self.get_n())
    }

    fn len(&self) -> usize {
        4
    }
}
