use super::{Body, Frame, PayloadSupport, Version, FLAG_METADATA, FLAG_RESUME};
use crate::utils::{RSocketResult, Writeable, DEFAULT_MIME_TYPE};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use std::time::Duration;

#[derive(Debug, PartialEq)]
pub struct Setup {
    version: Version,
    keepalive: u32,
    lifetime: u32,
    token: Option<Bytes>,
    mime_metadata: String,
    mime_data: String,
    metadata: Option<Bytes>,
    data: Option<Bytes>,
}

impl Writeable for Setup {
    fn len(&self) -> usize {
        let mut n: usize = 12;
        n += match &self.token {
            Some(v) => 2 + v.len(),
            None => 0,
        };
        n += 2 + self.mime_metadata.len() + self.mime_data.len();
        n += PayloadSupport::len(&self.metadata, &self.data);
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
        // TODO: remove string clone
        bf.put_u8(self.mime_metadata.len() as u8);
        bf.put(Bytes::from(self.mime_metadata.clone()));
        bf.put_u8(self.mime_data.len() as u8);
        bf.put(Bytes::from(self.mime_data.clone()));
        PayloadSupport::write(bf, self.get_metadata(), self.get_data());
    }
}

impl Setup {
    pub fn decode(flag: u16, b: &mut BytesMut) -> RSocketResult<Setup> {
        let major = b.get_u16();
        let minor = b.get_u16();
        let keepalive = b.get_u32();
        let lifetime = b.get_u32();
        let token: Option<Bytes> = if flag & FLAG_RESUME != 0 {
            let l = b.get_u16();
            Some(b.split_to(l as usize).to_bytes())
        } else {
            None
        };
        let mut len_mime: usize = b[0] as usize;
        b.advance(1);
        let mime_metadata = b.split_to(len_mime);
        len_mime = b[0] as usize;
        b.advance(1);
        let mime_data = b.split_to(len_mime);
        let (metadata, data) = PayloadSupport::read(flag, b);
        Ok(Setup {
            version: Version::new(major, minor),
            keepalive,
            lifetime,
            token,
            mime_metadata: String::from_utf8(mime_metadata.to_vec()).unwrap(),
            mime_data: String::from_utf8(mime_data.to_vec()).unwrap(),
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

    pub fn get_token(&self) -> Option<Bytes> {
        self.token.clone()
    }

    pub fn get_mime_metadata(&self) -> &str {
        &self.mime_metadata
    }

    pub fn get_mime_data(&self) -> &str {
        &self.mime_data
    }

    pub fn get_metadata(&self) -> &Option<Bytes> {
        &self.metadata
    }

    pub fn get_data(&self) -> &Option<Bytes> {
        &self.data
    }

    pub fn split(self) -> (Option<Bytes>, Option<Bytes>) {
        (self.data, self.metadata)
    }
}

pub struct SetupBuilder {
    stream_id: u32,
    flag: u16,
    value: Setup,
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
                mime_metadata: String::from(DEFAULT_MIME_TYPE),
                mime_data: String::from(DEFAULT_MIME_TYPE),
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
        self.flag |= FLAG_METADATA;
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
        self.flag |= FLAG_RESUME;
        self
    }

    pub fn set_mime_metadata(mut self, mime: &str) -> Self {
        if mime.len() > 256 {
            panic!("maximum mime length is 256");
        }
        self.value.mime_metadata = String::from(mime);
        self
    }

    pub fn set_mime_data(mut self, mime: &str) -> Self {
        if mime.len() > 256 {
            panic!("maximum mime length is 256");
        }
        self.value.mime_data = String::from(mime);
        self
    }
}
