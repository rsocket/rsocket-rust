use super::{Body, Frame, PayloadSupport, Version};
use crate::utils::{RSocketResult, Writeable, DEFAULT_MIME_TYPE};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub struct Setup {
    version: Version,
    keepalive: u32,
    lifetime: u32,
    token: Option<Bytes>,
    mime_metadata: Bytes,
    mime_data: Bytes,
    metadata: Option<Bytes>,
    data: Option<Bytes>,
}

pub struct SetupBuilder {
    stream_id: u32,
    flag: u16,
    value: Setup,
}

impl Setup {
    pub(crate) fn decode(flag: u16, b: &mut BytesMut) -> RSocketResult<Setup> {
        let major = b.get_u16();
        let minor = b.get_u16();
        let keepalive = b.get_u32();
        let lifetime = b.get_u32();
        let token: Option<Bytes> = if flag & Frame::FLAG_RESUME != 0 {
            let l = b.get_u16();
            Some(b.split_to(l as usize).freeze())
        } else {
            None
        };
        let mut len_mime: usize = b[0] as usize;
        b.advance(1);
        let mime_metadata = b.split_to(len_mime).freeze();
        len_mime = b[0] as usize;
        b.advance(1);
        let mime_data = b.split_to(len_mime).freeze();
        let (metadata, data) = PayloadSupport::read(flag, b);
        Ok(Setup {
            version: Version::new(major, minor),
            keepalive,
            lifetime,
            token,
            mime_metadata,
            mime_data,
            metadata,
            data,
        })
    }

    pub fn builder(stream_id: u32, flag: u16) -> SetupBuilder {
        SetupBuilder::new(stream_id, flag)
    }

    pub fn get_version(&self) -> Version {
        self.version
    }

    pub fn get_keepalive(&self) -> Duration {
        Duration::from_millis(u64::from(self.keepalive))
    }

    pub fn get_lifetime(&self) -> Duration {
        Duration::from_millis(u64::from(self.lifetime))
    }

    pub fn get_token(&self) -> Option<&Bytes> {
        match &self.token {
            Some(b) => Some(b),
            None => None,
        }
    }

    pub fn get_mime_metadata(&self) -> &str {
        std::str::from_utf8(self.mime_metadata.as_ref()).expect("Invalid UTF-8 bytes.")
    }

    pub fn get_mime_data(&self) -> &str {
        std::str::from_utf8(self.mime_data.as_ref()).expect("Invalid UTF-8 bytes.")
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

impl Writeable for Setup {
    fn len(&self) -> usize {
        let mut n: usize = 12;
        n += match &self.token {
            Some(v) => 2 + v.len(),
            None => 0,
        };
        n += 2 + self.mime_metadata.len() + self.mime_data.len();
        n += PayloadSupport::len(self.get_metadata(), self.get_data());
        n
    }

    fn write_to(&self, bf: &mut BytesMut) {
        self.version.write_to(bf);
        bf.put_u32(self.keepalive);
        bf.put_u32(self.lifetime);
        if let Some(b) = &self.token {
            bf.put_u16(b.len() as u16);
            bf.put(b.bytes());
        }
        bf.put_u8(self.mime_metadata.len() as u8);
        bf.put(self.mime_metadata.clone());
        bf.put_u8(self.mime_data.len() as u8);
        bf.put(self.mime_data.clone());
        PayloadSupport::write(bf, self.get_metadata(), self.get_data());
    }
}

impl SetupBuilder {
    fn new(stream_id: u32, flag: u16) -> SetupBuilder {
        SetupBuilder {
            stream_id,
            flag,
            value: Setup {
                version: Version::default(),
                keepalive: 30_000,
                lifetime: 90_000,
                token: None,
                mime_metadata: Bytes::from(DEFAULT_MIME_TYPE),
                mime_data: Bytes::from(DEFAULT_MIME_TYPE),
                metadata: None,
                data: None,
            },
        }
    }

    pub fn build(self) -> Frame {
        Frame::new(self.stream_id, Body::Setup(self.value), self.flag)
    }

    pub fn set_data(mut self, bs: Bytes) -> Self {
        self.value.data = Some(bs);
        self
    }

    pub fn set_metadata(mut self, bs: Bytes) -> Self {
        self.flag |= Frame::FLAG_METADATA;
        self.value.metadata = Some(bs);
        self
    }

    pub fn set_version(mut self, major: u16, minor: u16) -> Self {
        self.value.version = Version::new(major, minor);
        self
    }

    pub fn set_keepalive(mut self, duration: Duration) -> Self {
        self.value.keepalive = duration.as_millis() as u32;
        self
    }

    pub fn set_lifetime(mut self, duration: Duration) -> Self {
        self.value.lifetime = duration.as_millis() as u32;
        self
    }

    pub fn set_token(mut self, token: Bytes) -> Self {
        self.value.token = Some(token);
        self.flag |= Frame::FLAG_RESUME;
        self
    }

    pub fn set_mime_metadata<I>(mut self, mime: I) -> Self
    where
        I: Into<String>,
    {
        let mime = mime.into();
        assert!(mime.len() <= 256);
        self.value.mime_metadata = Bytes::from(mime);
        self
    }

    pub fn set_mime_data<I>(mut self, mime: I) -> Self
    where
        I: Into<String>,
    {
        let mime = mime.into();
        assert!(mime.len() <= 256);
        self.value.mime_data = Bytes::from(mime);
        self
    }
}
