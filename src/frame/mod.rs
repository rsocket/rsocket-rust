extern crate bytes;

use bytes::{BigEndian, BufMut, ByteOrder, Bytes, BytesMut};

mod cancel;
mod error;
mod keepalive;
mod lease;
mod metadata_push;
mod payload;
mod request_channel;
mod request_fnf;
mod request_n;
mod request_response;
mod request_stream;
mod resume;
mod resume_ok;
mod setup;
mod utils;

pub use cancel::Cancel;
pub use error::Error;
pub use keepalive::Keepalive;
pub use lease::Lease;
pub use metadata_push::MetadataPush;
pub use payload::Payload;
pub use request_channel::RequestChannel;
pub use request_fnf::RequestFNF;
pub use request_n::RequestN;
pub use request_response::RequestResponse;
pub use request_stream::RequestStream;
pub use resume::Resume;
pub use resume_ok::ResumeOK;
pub use setup::{Setup, SetupBuilder};
pub use utils::U24;

pub const FLAG_NEXT: u16 = 0x01 << 5;
pub const FLAG_COMPLETE: u16 = 0x01 << 6;
pub const FLAG_FOLLOW: u16 = 0x01 << 7;
pub const FLAG_METADATA: u16 = 0x01 << 8;
pub const FLAG_IGNORE: u16 = 0x01 << 9;
pub const FLAG_LEASE: u16 = FLAG_COMPLETE;
pub const FLAG_RESUME: u16 = FLAG_FOLLOW;
pub const FLAG_RESPOND: u16 = FLAG_FOLLOW;

pub const TYPE_SETUP: u16 = 0x01;
pub const TYPE_LEASE: u16 = 0x02;
pub const TYPE_KEEPALIVE: u16 = 0x03;
pub const TYPE_REQUEST_RESPONSE: u16 = 0x04;
pub const TYPE_REQUEST_FNF: u16 = 0x05;
pub const TYPE_REQUEST_STREAM: u16 = 0x06;
pub const TYPE_REQUEST_CHANNEL: u16 = 0x07;
pub const TYPE_REQUEST_N: u16 = 0x08;
pub const TYPE_CANCEL: u16 = 0x09;
pub const TYPE_PAYLOAD: u16 = 0x0A;
pub const TYPE_ERROR: u16 = 0x0B;
pub const TYPE_METADATA_PUSH: u16 = 0x0C;
pub const TYPE_RESUME: u16 = 0x0D;
pub const TYPE_RESUME_OK: u16 = 0x0E;

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
      Body::RequestResponse(v) => v.write_to(bf),
      Body::Keepalive(v) => v.write_to(bf),
      _ => unimplemented!(),
    }
  }

  fn len(&self) -> u32 {
    // header len
    let mut n: u32 = 6;
    match &self.body {
      Body::Setup(v) => n += v.len(),
      Body::RequestResponse(v) => n += v.len(),
      Body::Keepalive(v) => n += v.len(),
      _ => unimplemented!(),
    }
    n
  }
}

impl Frame {
  pub fn new(stream_id: u32, body: Body, flag: u16) -> Frame {
    Frame {
      stream_id,
      body,
      flag,
    }
  }

  pub fn decode(b: &mut Bytes) -> Option<Frame> {
    let sid = BigEndian::read_u32(b);
    b.advance(4);
    let n = BigEndian::read_u16(b);
    b.advance(2);
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
      TYPE_REQUEST_RESPONSE => {
        return Some(Frame {
          stream_id: sid,
          flag: flag,
          body: Body::RequestResponse(RequestResponse::decode(flag, b).unwrap()),
        })
      }
      TYPE_KEEPALIVE => {
        return Some(Frame {
          stream_id: sid,
          flag: flag,
          body: Body::Keepalive(Keepalive::decode(flag, b).unwrap()),
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
