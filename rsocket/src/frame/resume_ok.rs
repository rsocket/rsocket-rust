use bytes::{Buf, BufMut, Bytes, BytesMut};

use super::{Body, Frame};
use crate::error::RSocketError;
use crate::utils::Writeable;

#[derive(Debug, Eq, PartialEq)]
pub struct ResumeOK {
    position: u64,
}

pub struct ResumeOKBuilder {
    stream_id: u32,
    flag: u16,
    value: ResumeOK,
}

impl ResumeOKBuilder {
    fn new(stream_id: u32, flag: u16) -> ResumeOKBuilder {
        ResumeOKBuilder {
            stream_id,
            flag,
            value: ResumeOK { position: 0 },
        }
    }
    pub fn set_position(mut self, position: u64) -> Self {
        self.value.position = position;
        self
    }

    pub fn build(self) -> Frame {
        Frame::new(self.stream_id, Body::ResumeOK(self.value), self.flag)
    }
}

impl ResumeOK {
    pub(crate) fn decode(flag: u16, bf: &mut BytesMut) -> crate::Result<ResumeOK> {
        if bf.len() < 8 {
            Err(RSocketError::InCompleteFrame.into())
        } else {
            Ok(ResumeOK {
                position: bf.get_u64(),
            })
        }
    }

    pub fn builder(stream_id: u32, flag: u16) -> ResumeOKBuilder {
        ResumeOKBuilder::new(stream_id, flag)
    }

    pub fn get_position(&self) -> u64 {
        self.position
    }
}

impl Writeable for ResumeOK {
    fn write_to(&self, bf: &mut BytesMut) {
        bf.put_u64(self.get_position())
    }

    fn len(&self) -> usize {
        8
    }
}
