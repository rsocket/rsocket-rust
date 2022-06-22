use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::error::RSocketError;
use crate::utils::Writeable;

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
mod version;

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
pub use version::Version;

pub const REQUEST_MAX: u32 = 0x7FFF_FFFF; // 2147483647

pub(crate) const LEN_HEADER: usize = 6;

#[derive(Debug, Eq, PartialEq)]
pub enum Body {
    Setup(Setup),
    Lease(Lease),
    Keepalive(Keepalive),
    RequestFNF(RequestFNF),
    RequestResponse(RequestResponse),
    RequestStream(RequestStream),
    RequestChannel(RequestChannel),
    RequestN(RequestN),
    Cancel(),
    Payload(Payload),
    Error(Error),
    MetadataPush(MetadataPush),
    Resume(Resume),
    ResumeOK(ResumeOK),
}

#[derive(Debug, Eq, PartialEq)]
pub struct Frame {
    pub(crate) stream_id: u32,
    pub(crate) body: Body,
    pub(crate) flag: u16,
}

impl Frame {
    pub const FLAG_NEXT: u16 = 0x01 << 5;
    pub const FLAG_COMPLETE: u16 = 0x01 << 6;
    pub const FLAG_FOLLOW: u16 = 0x01 << 7;
    pub const FLAG_METADATA: u16 = 0x01 << 8;
    pub const FLAG_IGNORE: u16 = 0x01 << 9;
    pub const FLAG_LEASE: u16 = Self::FLAG_COMPLETE;
    pub const FLAG_RESUME: u16 = Self::FLAG_FOLLOW;
    pub const FLAG_RESPOND: u16 = Self::FLAG_FOLLOW;

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
}

impl Writeable for Frame {
    fn write_to(&self, bf: &mut BytesMut) {
        bf.put_u32(self.stream_id);
        bf.put_u16((to_frame_type(&self.body) << 10) | self.flag);
        match &self.body {
            Body::Setup(v) => v.write_to(bf),
            Body::RequestResponse(v) => v.write_to(bf),
            Body::RequestStream(v) => v.write_to(bf),
            Body::RequestChannel(v) => v.write_to(bf),
            Body::RequestFNF(v) => v.write_to(bf),
            Body::RequestN(v) => v.write_to(bf),
            Body::MetadataPush(v) => v.write_to(bf),
            Body::Keepalive(v) => v.write_to(bf),
            Body::Payload(v) => v.write_to(bf),
            Body::Lease(v) => v.write_to(bf),
            Body::Error(v) => v.write_to(bf),
            Body::Cancel() => (),
            Body::ResumeOK(v) => v.write_to(bf),
            Body::Resume(v) => v.write_to(bf),
        }
    }

    fn len(&self) -> usize {
        // header len
        LEN_HEADER
            + match &self.body {
                Body::Setup(v) => v.len(),
                Body::RequestResponse(v) => v.len(),
                Body::RequestStream(v) => v.len(),
                Body::RequestChannel(v) => v.len(),
                Body::RequestFNF(v) => v.len(),
                Body::RequestN(v) => v.len(),
                Body::MetadataPush(v) => v.len(),
                Body::Keepalive(v) => v.len(),
                Body::Payload(v) => v.len(),
                Body::Lease(v) => v.len(),
                Body::Cancel() => 0,
                Body::Error(v) => v.len(),
                Body::ResumeOK(v) => v.len(),
                Body::Resume(v) => v.len(),
            }
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

    pub fn decode(b: &mut BytesMut) -> crate::Result<Frame> {
        if b.len() < LEN_HEADER {
            return Err(RSocketError::InCompleteFrame.into());
        }
        let sid = b.get_u32();
        let n = b.get_u16();
        let (flag, kind) = (n & 0x03FF, (n & 0xFC00) >> 10);
        let body = match kind {
            Self::TYPE_SETUP => Setup::decode(flag, b).map(Body::Setup),
            Self::TYPE_REQUEST_RESPONSE => {
                RequestResponse::decode(flag, b).map(Body::RequestResponse)
            }
            Self::TYPE_REQUEST_STREAM => RequestStream::decode(flag, b).map(Body::RequestStream),
            Self::TYPE_REQUEST_CHANNEL => RequestChannel::decode(flag, b).map(Body::RequestChannel),
            Self::TYPE_REQUEST_FNF => RequestFNF::decode(flag, b).map(Body::RequestFNF),
            Self::TYPE_REQUEST_N => RequestN::decode(flag, b).map(Body::RequestN),
            Self::TYPE_METADATA_PUSH => MetadataPush::decode(flag, b).map(Body::MetadataPush),
            Self::TYPE_KEEPALIVE => Keepalive::decode(flag, b).map(Body::Keepalive),
            Self::TYPE_PAYLOAD => Payload::decode(flag, b).map(Body::Payload),
            Self::TYPE_LEASE => Lease::decode(flag, b).map(Body::Lease),
            Self::TYPE_CANCEL => Ok(Body::Cancel()),
            Self::TYPE_ERROR => Error::decode(flag, b).map(Body::Error),
            Self::TYPE_RESUME_OK => ResumeOK::decode(flag, b).map(Body::ResumeOK),
            Self::TYPE_RESUME => Resume::decode(flag, b).map(Body::Resume),
            typ => unreachable!("invalid frame type {}!", typ),
        };
        body.map(|it| Frame::new(sid, it, flag))
    }

    pub(crate) fn is_followable_or_payload(&self) -> (bool, bool) {
        match &self.body {
            Body::RequestFNF(_) => (true, false),
            Body::RequestResponse(_) => (true, false),
            Body::RequestStream(_) => (true, false),
            Body::RequestChannel(_) => (true, false),
            Body::Payload(_) => (true, true),
            _ => (false, false),
        }
    }

    pub fn get_body(self) -> Body {
        self.body
    }

    pub fn get_body_ref(&self) -> &Body {
        &self.body
    }

    pub fn get_flag(&self) -> u16 {
        self.flag
    }

    pub fn get_stream_id(&self) -> u32 {
        self.stream_id
    }

    pub fn has_next(&self) -> bool {
        self.flag & Self::FLAG_NEXT != 0
    }

    pub fn has_complete(&self) -> bool {
        self.flag & Self::FLAG_COMPLETE != 0
    }
}

#[inline]
fn to_frame_type(body: &Body) -> u16 {
    match body {
        Body::Setup(_) => Frame::TYPE_SETUP,
        Body::Lease(_) => Frame::TYPE_LEASE,
        Body::Keepalive(_) => Frame::TYPE_KEEPALIVE,
        Body::RequestResponse(_) => Frame::TYPE_REQUEST_RESPONSE,
        Body::RequestFNF(_) => Frame::TYPE_REQUEST_FNF,
        Body::RequestStream(_) => Frame::TYPE_REQUEST_STREAM,
        Body::RequestChannel(_) => Frame::TYPE_REQUEST_CHANNEL,
        Body::RequestN(_) => Frame::TYPE_REQUEST_N,
        Body::Cancel() => Frame::TYPE_CANCEL,
        Body::Payload(_) => Frame::TYPE_PAYLOAD,
        Body::Error(_) => Frame::TYPE_ERROR,
        Body::MetadataPush(_) => Frame::TYPE_METADATA_PUSH,
        Body::Resume(_) => Frame::TYPE_RESUME,
        Body::ResumeOK(_) => Frame::TYPE_RESUME_OK,
    }
}
