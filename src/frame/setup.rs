extern crate bytes;

use crate::frame::{Body, Frame, PayloadSupport, Writeable, FLAG_METADATA, FLAG_RESUME, U24};
use crate::mime::MIME_BINARY;
use bytes::{BigEndian, BufMut, ByteOrder, Bytes, BytesMut};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Version {
  major: u16,
  minor: u16,
}

impl Version {
  pub fn default() -> Version {
    Version { major: 1, minor: 0 }
  }

  fn new(major: u16, minor: u16) -> Version {
    Version { major, minor }
  }

  pub fn get_major(&self) -> u16 {
    self.major.clone()
  }

  pub fn get_minor(&self) -> u16 {
    self.minor.clone()
  }

  pub fn write_to(&self, bf: &mut BytesMut) {
    bf.put_u16_be(self.major);
    bf.put_u16_be(self.minor);
  }
}

#[derive(Debug, Clone)]
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
  fn len(&self) -> u32 {
    let mut n: u32 = 12;
    n += match &self.token {
      Some(v) => 2 + (v.len() as u32),
      None => 0,
    };
    n += 1;
    n += self.mime_metadata.len() as u32;
    n += 1;
    n += self.mime_data.len() as u32;
    n += PayloadSupport::len(&self.metadata, &self.data);
    n
  }

  fn write_to(&self, bf: &mut BytesMut) {
    self.version.write_to(bf);
    bf.put_u32_be(self.keepalive);
    bf.put_u32_be(self.lifetime);
    match &self.token {
      Some(v) => {
        bf.put_u16_be(v.len() as u16);
        bf.put(v);
      }
      None => (),
    }
    bf.put_u8(self.mime_metadata.len() as u8);
    bf.put(&self.mime_metadata);
    bf.put_u8(self.mime_data.len() as u8);
    bf.put(&self.mime_data);
    PayloadSupport::write(bf, &self.metadata, &self.data);
  }
}

impl Setup {
  pub fn decode(flag: u16, b: &mut BytesMut) -> Option<Setup> {
    let major = BigEndian::read_u16(b);
    b.advance(2);
    let minor = BigEndian::read_u16(b);
    b.advance(2);
    let keepalive = BigEndian::read_u32(b);
    b.advance(4);
    let lifetime = BigEndian::read_u32(b);
    b.advance(4);
    let token: Option<Bytes> = if flag & FLAG_RESUME != 0 {
      let l = BigEndian::read_u16(b);
      b.advance(2);
      Some(Bytes::from(b.split_to(l as usize)))
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
    Some(Setup {
      version: Version::new(major, minor),
      keepalive: keepalive,
      lifetime: lifetime,
      token: token,
      mime_metadata: String::from_utf8(mime_metadata.to_vec()).unwrap(),
      mime_data: String::from_utf8(mime_data.to_vec()).unwrap(),
      metadata: metadata,
      data: data,
    })
  }

  pub fn builder(stream_id: u32, flag: u16) -> SetupBuilder {
    SetupBuilder::new(stream_id, flag)
  }

  pub fn get_version(&self) -> &Version {
    &self.version
  }

  pub fn get_keepalive(&self) -> Duration {
    Duration::from_millis(self.keepalive as u64)
  }

  pub fn get_lifetime(&self) -> Duration {
    Duration::from_millis(self.lifetime as u64)
  }

  pub fn get_token(&self) -> Option<Bytes> {
    self.token.clone()
  }

  pub fn get_mime_metadata(&self) -> &String {
    &self.mime_metadata
  }

  pub fn get_mime_data(&self) -> &String {
    &self.mime_data
  }

  pub fn get_metadata(&self) -> Option<Bytes> {
    self.metadata.clone()
  }

  pub fn get_data(&self) -> Option<Bytes> {
    self.data.clone()
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
      stream_id: stream_id,
      flag: flag,
      value: Setup {
        version: Version::default(),
        keepalive: 30_000,
        lifetime: 90_000,
        token: None,
        mime_metadata: String::from(MIME_BINARY),
        mime_data: String::from(MIME_BINARY),
        metadata: None,
        data: None,
      },
    }
  }

  pub fn build(&mut self) -> Frame {
    Frame::new(self.stream_id, Body::Setup(self.value.clone()), self.flag)
  }

  pub fn set_data(&mut self, bs: Bytes) -> &mut SetupBuilder {
    self.value.data = Some(bs);
    self
  }

  pub fn set_metadata(&mut self, bs: Bytes) -> &mut SetupBuilder {
    self.flag |= FLAG_METADATA;
    self.value.metadata = Some(bs);
    self
  }

  pub fn set_version(&mut self, major: u16, minor: u16) -> &mut SetupBuilder {
    self.value.version = Version::new(major, minor);
    self
  }

  pub fn set_keepalive(&mut self, duration: Duration) -> &mut SetupBuilder {
    self.value.keepalive = duration.as_millis() as u32;
    self
  }

  pub fn set_lifetime(&mut self, duration: Duration) -> &mut SetupBuilder {
    self.value.lifetime = duration.as_millis() as u32;
    self
  }

  pub fn set_token(&mut self, token: Bytes) -> &mut SetupBuilder {
    self.value.token = Some(token);
    self.flag |= FLAG_RESUME;
    self
  }

  pub fn set_mime_metadata(&mut self, mime: &str) -> &mut SetupBuilder {
    if mime.len() > 256 {
      panic!("maximum mime length is 256");
    }
    self.value.mime_metadata = String::from(mime);
    self
  }

  pub fn set_mime_data(&mut self, mime: &str) -> &mut SetupBuilder {
    if mime.len() > 256 {
      panic!("maximum mime length is 256");
    }
    self.value.mime_data = String::from(mime);
    self
  }
}
