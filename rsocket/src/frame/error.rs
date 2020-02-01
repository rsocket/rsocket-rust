use super::{Body, Frame, Writeable};
use crate::misc::RSocketResult;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::fmt;

#[derive(Debug, PartialEq)]
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
    pub fn decode(flag: u16, bf: &mut BytesMut) -> RSocketResult<Error> {
        let code = bf.get_u32();
        let d: Option<Bytes> = if !bf.is_empty() {
            Some(bf.to_bytes())
        } else {
            None
        };
        Ok(Error { code, data: d })
    }

    pub fn builder(stream_id: u32, flag: u16) -> ErrorBuilder {
        ErrorBuilder::new(stream_id, flag)
    }

    pub fn get_data_utf8(&self) -> String {
        match self.get_data() {
            Some(b) => String::from_utf8(b.to_vec()).unwrap(),
            None => String::from(""),
        }
    }

    pub fn get_data(&self) -> &Option<Bytes> {
        &self.data
    }

    pub fn get_code(&self) -> u32 {
        self.code
    }
}

impl Writeable for Error {
    fn write_to(&self, bf: &mut BytesMut) {
        bf.put_u32(self.code);
        match &self.data {
            Some(v) => bf.put(v.bytes()),
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
