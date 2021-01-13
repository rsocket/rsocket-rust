use std::io::{Error, ErrorKind};

use bytes::{Buf, BytesMut};
use rsocket_rust::frame::Frame;
use rsocket_rust::utils::{u24, Writeable};
use tokio_util::codec::{Decoder, Encoder};

pub struct LengthBasedFrameCodec;

const LEN_BYTES: usize = 3;

impl Decoder for LengthBasedFrameCodec {
    type Item = Frame;
    type Error = Error;

    fn decode(&mut self, buf: &mut BytesMut) -> Result<Option<Self::Item>, Self::Error> {
        let actual = buf.len();
        if actual < LEN_BYTES {
            return Ok(None);
        }
        let l = u24::read(buf).into();
        if actual < LEN_BYTES + l {
            return Ok(None);
        }
        buf.advance(LEN_BYTES);
        let mut bb = buf.split_to(l);
        match Frame::decode(&mut bb) {
            Ok(v) => Ok(Some(v)),
            Err(_e) => Err(Error::from(ErrorKind::InvalidInput)),
        }
    }
}

impl Encoder<Frame> for LengthBasedFrameCodec {
    type Error = Error;
    fn encode(&mut self, item: Frame, buf: &mut BytesMut) -> Result<(), Self::Error> {
        let l = item.len();
        buf.reserve(LEN_BYTES + l);
        u24::from(l).write_to(buf);
        item.write_to(buf);
        Ok(())
    }
}
