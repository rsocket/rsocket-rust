extern crate bytes;

use bytes::{BigEndian, BufMut, ByteOrder, Bytes, BytesMut};

use std::time::Duration;
const FLAG_NEXT: u16 = 0x01 << 5;
const FLAG_COMPLETE: u16 = 0x01 << 6;
const FLAG_FOLLOW: u16 = 0x01 << 7;
const FLAG_METADATA: u16 = 0x01 << 8;
const FLAG_IGNORE: u16 = 0x01 << 9;
const FLAG_LEASE: u16 = FLAG_COMPLETE;
const FLAG_RESUME: u16 = FLAG_FOLLOW;
const FLAG_RESPOND: u16 = FLAG_FOLLOW;

const TYPE_SETUP: u16 = 0x01;
const TYPE_LEASE: u16 = 0x02;
const TYPE_KEEPALIVE: u16 = 0x03;
const TYPE_REQUEST_RESPONSE: u16 = 0x04;
const TYPE_REQUEST_FNF: u16 = 0x05;
const TYPE_REQUEST_STREAM: u16 = 0x06;
const TYPE_REQUEST_CHANNEL: u16 = 0x07;
const TYPE_REQUEST_N: u16 = 0x08;
const TYPE_CANCEL: u16 = 0x09;
const TYPE_PAYLOAD: u16 = 0x0A;
const TYPE_ERROR: u16 = 0x0B;
const TYPE_METADATA_PUSH: u16 = 0x0C;
const TYPE_RESUME: u16 = 0x0D;
const TYPE_RESUME_OK: u16 = 0x0E;

pub const MIME_BINARY: &str = "application/binary";

pub trait Writeable {
  fn write_to(&self, bf: &mut BytesMut);
  fn len(&self) -> u32;
}

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
  stream_id: u32,
  body: Body,
  flag: u16,
}

impl Writeable for Frame {

  fn write_to(&self, bf: &mut BytesMut) {
    bf.put_u32_be(self.stream_id);
    bf.put_u16_be((to_frame_type(&self.body) << 10) | self.flag);
    match &self.body {
      Body::Setup(v) => v.write_to(bf),
      _ => unimplemented!(),
    }
  }

  fn len(&self) -> u32 {
    // header len
    let mut n: u32 = 6;
    match &self.body {
      Body::Setup(v) => n += v.len(),
      _ => unimplemented!(),
    }
    n
  }
}

impl Frame {
  pub fn decode(b: &mut Bytes) -> Option<Frame> {
    let hd = b.split_to(6);
    let v = &hd.to_vec();
    let sid = BigEndian::read_u32(v);
    let n = (v[4] as u16) << 8 + (v[5] as u16);
    let flag = n & 0x03FF;
    let t = (n & 0xFC00) >> 10;
    // println!("**** type={}, sid={}, flag={}", t, sid, flag);
    match t {
      TYPE_SETUP => {
        return Some(Frame {
          stream_id: sid,
          flag: flag,
          body: Body::Setup(Setup::decode(flag, b).unwrap()),
        })
      }
      _ => unimplemented!(),
    }
  }

  pub fn get_body(&self) -> &Body {
    &self.body
  }

  pub fn get_flag(&self) -> u16 {
    self.flag.clone()
  }

  pub fn get_stream_id(&self) -> u32 {
    self.stream_id.clone()
  }

}

fn to_frame_type(body: &Body) -> u16 {
  return match body {
    Body::Setup(_) => TYPE_SETUP,
    Body::Lease(_) => TYPE_LEASE,
    Body::Keepalive(_) => TYPE_KEEPALIVE,
    Body::RequestResponse(_) => TYPE_REQUEST_RESPONSE,
    Body::RequestFNF(_) => TYPE_REQUEST_FNF,
    Body::RequestStream(_) => TYPE_REQUEST_STREAM,
    Body::RequestChannel(_) => TYPE_REQUEST_CHANNEL,
    Body::RequestN(_) => TYPE_REQUEST_N,
    Body::Cancel(_) => TYPE_CANCEL,
    Body::Payload(_) => TYPE_PAYLOAD,
    Body::Error(_) => TYPE_ERROR,
    Body::MetadataPush(_) => TYPE_METADATA_PUSH,
    Body::Resume(_) => TYPE_RESUME,
    Body::ResumeOK(_) => TYPE_RESUME_OK,
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
    match &self.token {
      Some(v) => n += 2 + (v.len() as u32),
      None => (),
    }
    n += 1;
    n += self.mime_metadata.len() as u32;
    n += 1;
    n += self.mime_data.len() as u32;
    match &self.metadata {
      Some(v) => {
        n += 3;
        n += v.len() as u32;
      }
      None => (),
    }
    match &self.data {
      Some(v) => n += v.len() as u32,
      None => (),
    }
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

    match &self.metadata {
      Some(v) => {
        let l = v.len();
        bf.put_u8((0xFF & (l >> 16)) as u8);
        bf.put_u8((0xFF & (l >> 8)) as u8);
        bf.put_u8((0xFF & l) as u8);
        bf.put(v);
      }
      None => (),
    }
    match &self.data {
      Some(v) => bf.put(v),
      None => (),
    }
  }

}

impl Setup {

  fn decode(flag: u16, b: &mut Bytes) -> Option<Setup> {
    let major = BigEndian::read_u16(b);
    b.advance(2);
    let minor = BigEndian::read_u16(b);
    b.advance(2);
    let keepalive = BigEndian::read_u32(b);
    b.advance(4);
    let lifetime = BigEndian::read_u32(b);
    b.advance(4);
    let mut token: Option<Bytes> = None;
    if flag & FLAG_RESUME != 0 {
      let l = BigEndian::read_u16(b);
      b.advance(2);
      token = Some(Bytes::from(b.split_to(l as usize)));
    }
    let mut len_mime: usize = b[0] as usize;
    b.advance(1);
    let mime_metadata = b.split_to(len_mime);
    len_mime = b[0] as usize;
    b.advance(1);
    let mime_data = b.split_to(len_mime);

    let mut metadata: Option<Bytes> = None;
    if flag & FLAG_METADATA != 0 {
      let bar = b.split_to(3);
      let mut l: u32 = 0;
      l += (bar[0] as u32) << 16;
      l += (bar[1] as u32) << 8;
      l += bar[2] as u32;
      metadata = Some(b.split_to(l as usize));
    }
    let mut data: Option<Bytes> = None;
    if !b.is_empty() {
      data = Some(Bytes::from(b.to_vec()));
    }
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
        metadata: None,
        data: None,
      },
    }
  }

  pub fn build(&mut self) -> Frame {
    Frame {
      stream_id: self.stream_id,
      body: Body::Setup(self.setup.clone()),
      flag: self.flag,
    }
  }

  pub fn set_data(&mut self, bs: Bytes) -> &mut SetupBuilder {
    self.setup.data = Some(bs);
    self
  }

  pub fn set_metadata(&mut self, bs: Bytes) -> &mut SetupBuilder {
    self.flag |= FLAG_METADATA;
    self.setup.metadata = Some(bs);
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

  pub fn set_mime_metadata(&mut self, mime: &str) -> &mut SetupBuilder {
    self.setup.mime_metadata = String::from(mime);
    self
  }

  pub fn set_mime_data(&mut self, mime: &str) -> &mut SetupBuilder {
    self.setup.mime_data = String::from(mime);
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
