use bytes::{Buf, BufMut, Bytes, BytesMut};

use super::{Body, Frame, Version};
use crate::error::RSocketError;
use crate::utils::Writeable;

#[derive(Debug, Eq, PartialEq)]
pub struct Resume {
    version: Version,
    token: Option<Bytes>,
    last_received_server_position: u64,
    first_available_client_position: u64,
}

pub struct ResumeBuilder {
    stream_id: u32,
    flag: u16,
    inner: Resume,
}

impl Resume {
    fn new() -> Resume {
        Resume {
            version: Version::default(),
            token: None,
            last_received_server_position: 0,
            first_available_client_position: 0,
        }
    }

    pub(crate) fn decode(flag: u16, b: &mut BytesMut) -> crate::Result<Resume> {
        if b.len() < 6 {
            return Err(RSocketError::InCompleteFrame.into());
        }
        let major = b.get_u16();
        let minor = b.get_u16();
        let token_size = b.get_u16() as usize;
        let token = if token_size > 0 {
            if b.len() < token_size {
                return Err(RSocketError::InCompleteFrame.into());
            }
            Some(b.split_to(token_size).freeze())
        } else {
            None
        };
        if b.len() < 16 {
            return Err(RSocketError::InCompleteFrame.into());
        }
        let last_received_server_position = b.get_u64();
        let first_available_client_position = b.get_u64();
        Ok(Resume {
            version: Version::new(major, minor),
            token,
            last_received_server_position,
            first_available_client_position,
        })
    }

    pub fn builder(stream_id: u32, flag: u16) -> ResumeBuilder {
        ResumeBuilder::new(stream_id, flag)
    }

    pub fn get_version(&self) -> Version {
        self.version
    }

    pub fn get_token(&self) -> &Option<Bytes> {
        &self.token
    }

    pub fn get_last_received_server_position(&self) -> u64 {
        self.last_received_server_position
    }

    pub fn get_first_available_client_position(&self) -> u64 {
        self.first_available_client_position
    }
}

impl ResumeBuilder {
    fn new(stream_id: u32, flag: u16) -> ResumeBuilder {
        ResumeBuilder {
            stream_id,
            flag,
            inner: Resume::new(),
        }
    }

    pub fn set_token(mut self, token: Bytes) -> Self {
        self.inner.token = Some(token);
        self
    }

    pub fn set_last_received_server_position(mut self, position: u64) -> Self {
        self.inner.last_received_server_position = position;
        self
    }

    pub fn set_first_available_client_position(mut self, position: u64) -> Self {
        self.inner.first_available_client_position = position;
        self
    }

    pub fn build(self) -> Frame {
        Frame {
            stream_id: self.stream_id,
            flag: self.flag,
            body: Body::Resume(self.inner),
        }
    }
}

impl Writeable for Resume {
    fn write_to(&self, bf: &mut BytesMut) {
        self.version.write_to(bf);
        if let Some(b) = self.get_token() {
            bf.put_u16(b.len() as u16);
            bf.extend_from_slice(b);
        }
        bf.put_u64(self.get_last_received_server_position());
        bf.put_u64(self.get_first_available_client_position());
    }

    fn len(&self) -> usize {
        let mut size: usize = 22;
        if let Some(b) = self.get_token() {
            size += b.len();
        }
        size
    }
}
