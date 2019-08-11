extern crate bytes;

use bytes::{BigEndian, BufMut, Bytes, BytesMut};
use std::time::Duration;

pub const MIME_BINARY: &str = "application/binary";

#[derive(Debug)]
pub enum Body {
  Setup(Setup),
  Lease(Lease),
  Keepalive(Keepalive),
  RequestFNF(RequestFNF),
  RequestResponse(RequestResponse),
  RequestStream(RequestStream),
  RequestChannel(RequestChannel),
  RequestN(RequestN),
  Cancel(Cancel),
  Payload(Payload),
  Error(Error),
  MetadataPush(MetadataPush),
  Resume(Resume),
  ResumeOK(ResumeOK),
}

#[derive(Debug)]
pub struct Frame {
  pub stream_id: u32,
  pub body: Body,
  pub flag: u16,
}

impl Frame {

  pub fn new(stream_id: u32, body: Body, flag: u16) -> Frame {
    Frame {
      stream_id,
      body,
      flag,
    }
  }

  pub fn to_bytes(&self) -> Bytes {
    let mut bm = BytesMut::new();
    bm.put_u32_be(self.stream_id);
    bm.put_u16_be((to_frame_type(&self.body) << 10) | self.flag);
    bm.freeze().clone()
  }

}

fn to_frame_type(body: &Body) -> u16 {
  return match body {
    Body::Setup(_) => 0x01,
    Body::Lease(_) => 0x02,
    Body::Keepalive(_) => 0x03,
    Body::RequestResponse(_) => 0x04,
    Body::RequestFNF(_) => 0x05,
    Body::RequestStream(_) => 0x06,
    Body::RequestChannel(_) => 0x07,
    Body::RequestN(_) => 0x08,
    Body::Cancel(_) => 0x09,
    Body::Payload(_) => 0x0A,
    Body::Error(_) => 0x0B,
    Body::MetadataPush(_) => 0x0C,
    Body::Resume(_) => 0x0D,
    Body::ResumeOK(_) => 0x0E,
  };
}

#[derive(Debug, Clone)]
pub struct Version {
  major: u16,
  minor: u16,
}

impl Version {
  fn new(major: u16, minor: u16) -> Version {
    Version { major, minor }
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
  meatadata: Option<Bytes>,
  data: Option<Bytes>,
}

impl Setup {
  pub fn builder(stream_id: u32, flag: u16) -> SetupBuilder {
    SetupBuilder::new(stream_id, flag)
  }
}

pub struct SetupBuilder {
  stream_id: u32,
  flag: u16,
  setup: Setup,
}

impl SetupBuilder {
  fn new(stream_id: u32, flag: u16) -> SetupBuilder {
    SetupBuilder {
      stream_id: stream_id,
      flag: flag,
      setup: Setup {
        version: Version::new(1, 0),
        keepalive: 30_000,
        lifetime: 90_000,
        token: None,
        mime_metadata: String::from(MIME_BINARY),
        mime_data: String::from(MIME_BINARY),
        meatadata: None,
        data: None,
      },
    }
  }

  pub fn build(&mut self) -> Frame {
    Frame::new(self.stream_id, Body::Setup(self.setup.clone()), self.flag)
  }

  pub fn set_data(&mut self, bs: Bytes) -> &mut SetupBuilder {
    self.setup.data = Some(bs);
    self
  }

  pub fn set_metadata(&mut self, bs: Bytes) -> &mut SetupBuilder {
    self.setup.meatadata = Some(bs);
    self
  }

  pub fn set_version(&mut self, major: u16, minor: u16) -> &mut SetupBuilder {
    self.setup.version = Version::new(major, minor);
    self
  }

  pub fn set_keepalive(&mut self, duration: Duration) -> &mut SetupBuilder {
    self.setup.keepalive = duration.as_millis() as u32;
    self
  }

  pub fn set_lifetime(&mut self, duration: Duration) -> &mut SetupBuilder {
    self.setup.lifetime = duration.as_millis() as u32;
    self
  }

  pub fn set_token(&mut self, token: Bytes) -> &mut SetupBuilder {
    self.setup.token = Some(token);
    self
  }

  pub fn set_mime_metadata(&mut self, mime: String) -> &mut SetupBuilder {
    self.setup.mime_metadata = mime;
    self
  }

  pub fn set_mime_data(&mut self, mime: String) -> &mut SetupBuilder {
    self.setup.mime_data = mime;
    self
  }

}

#[derive(Debug)]
pub struct Lease {}

#[derive(Debug)]
pub struct Keepalive {}

#[derive(Debug)]
pub struct RequestFNF {}

#[derive(Debug)]
pub struct RequestResponse {}

#[derive(Debug)]
pub struct RequestStream {}

#[derive(Debug)]
pub struct RequestChannel {}

#[derive(Debug)]
pub struct RequestN {}

#[derive(Debug)]
pub struct Cancel {}

#[derive(Debug)]
pub struct Payload {}

#[derive(Debug)]
pub struct Error {}

#[derive(Debug)]
pub struct MetadataPush {}

#[derive(Debug)]
pub struct Resume {}

#[derive(Debug)]
pub struct ResumeOK {}
