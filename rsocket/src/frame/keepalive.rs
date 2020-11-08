use super::utils::too_short;
use super::{Body, Frame};
use crate::utils::Writeable;
use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug, PartialEq)]
pub struct Keepalive {
    last_received_position: u64,
    data: Option<Bytes>,
}

pub struct KeepaliveBuilder {
    stream_id: u32,
    flag: u16,
    keepalive: Keepalive,
}

impl KeepaliveBuilder {
    fn new(stream_id: u32, flag: u16) -> KeepaliveBuilder {
        KeepaliveBuilder {
            stream_id,
            flag,
            keepalive: Keepalive {
                last_received_position: 0,
                data: None,
            },
        }
    }

    pub fn set_data(mut self, data: Bytes) -> Self {
        self.keepalive.data = Some(data);
        self
    }

    pub fn set_last_received_position(mut self, position: u64) -> Self {
        self.keepalive.last_received_position = position;
        self
    }

    pub fn build(self) -> Frame {
        Frame::new(self.stream_id, Body::Keepalive(self.keepalive), self.flag)
    }
}

impl Keepalive {
    pub(crate) fn decode(flag: u16, bf: &mut BytesMut) -> crate::Result<Keepalive> {
        if bf.len() < 8 {
            return too_short(8);
        }
        let position = bf.get_u64();
        let data = if bf.is_empty() {
            None
        } else {
            Some(Bytes::from(bf.to_vec()))
        };
        Ok(Keepalive {
            last_received_position: position,
            data,
        })
    }

    pub fn builder(stream_id: u32, flag: u16) -> KeepaliveBuilder {
        KeepaliveBuilder::new(stream_id, flag)
    }

    pub fn get_last_received_position(&self) -> u64 {
        self.last_received_position
    }

    pub fn get_data(&self) -> Option<&Bytes> {
        match &self.data {
            Some(b) => Some(b),
            None => None,
        }
    }

    pub fn split(self) -> (Option<Bytes>, Option<Bytes>) {
        (self.data, None)
    }
}

impl Writeable for Keepalive {
    fn write_to(&self, bf: &mut BytesMut) {
        bf.put_u64(self.last_received_position);
        match &self.data {
            Some(v) => bf.put(v.bytes()),
            None => (),
        }
    }

    fn len(&self) -> usize {
        8 + match &self.data {
            Some(v) => v.len(),
            None => 0,
        }
    }
}
