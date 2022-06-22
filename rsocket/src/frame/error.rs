use std::fmt;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use super::{Body, Frame};
use crate::error::RSocketError;
use crate::utils::Writeable;

#[derive(Debug, Eq, PartialEq)]
pub struct Error {
    code: u32,
    data: Option<Bytes>,
}

pub struct ErrorBuilder {
    stream_id: u32,
    flag: u16,
    value: Error,
}

impl ErrorBuilder {
    fn new(stream_id: u32, flag: u16) -> ErrorBuilder {
        ErrorBuilder {
            stream_id,
            flag,
            value: Error {
                code: 0,
                data: None,
            },
        }
    }

    pub fn set_code(mut self, code: u32) -> Self {
        self.value.code = code;
        self
    }

    pub fn set_data(mut self, data: Bytes) -> Self {
        self.value.data = Some(data);
        self
    }

    pub fn build(self) -> Frame {
        Frame::new(self.stream_id, Body::Error(self.value), self.flag)
    }
}

impl Error {
    pub(crate) fn decode(flag: u16, bf: &mut BytesMut) -> crate::Result<Error> {
        if bf.len() < 4 {
            Err(RSocketError::InCompleteFrame.into())
        } else {
            let code = bf.get_u32();
            let data: Option<Bytes> = if !bf.is_empty() {
                Some(bf.split().freeze())
            } else {
                None
            };
            Ok(Error { code, data })
        }
    }

    pub fn builder(stream_id: u32, flag: u16) -> ErrorBuilder {
        ErrorBuilder::new(stream_id, flag)
    }

    pub fn get_data_utf8(&self) -> Option<&str> {
        match &self.data {
            Some(b) => std::str::from_utf8(b).ok(),
            None => None,
        }
    }

    pub fn get_data(&self) -> Option<&Bytes> {
        self.data.as_ref()
    }

    pub fn get_code(&self) -> u32 {
        self.code
    }
}

impl Writeable for Error {
    fn write_to(&self, bf: &mut BytesMut) {
        bf.put_u32(self.code);
        match &self.data {
            Some(v) => bf.extend_from_slice(v),
            None => (),
        }
    }

    fn len(&self) -> usize {
        4 + match &self.data {
            Some(v) => v.len(),
            None => 0,
        }
    }
}
