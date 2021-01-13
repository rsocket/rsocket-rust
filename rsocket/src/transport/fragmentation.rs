use std::collections::LinkedList;

use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::frame::{self, Body, Frame};
use crate::payload::Payload;

pub(crate) const MIN_MTU: usize = 64;

pub(crate) struct Joiner {
    inner: LinkedList<Frame>,
}

#[derive(Debug, Clone)]
pub(crate) struct Splitter {
    mtu: usize,
}

impl Splitter {
    pub(crate) fn new(mtu: usize) -> Splitter {
        assert!(mtu > frame::LEN_HEADER, "mtu is too small!");
        Splitter { mtu }
    }

    pub(crate) fn cut(&self, input: Payload, skip: usize) -> impl Iterator<Item = Payload> {
        let (data, meta) = input.split();
        SplitterIter {
            mtu: self.mtu,
            skip,
            data,
            meta,
        }
    }
}

struct SplitterIter {
    mtu: usize,
    skip: usize,
    data: Option<Bytes>,
    meta: Option<Bytes>,
}

impl Iterator for SplitterIter {
    type Item = Payload;

    fn next(&mut self) -> Option<Payload> {
        if self.meta.is_none() && self.data.is_none() {
            return None;
        }
        let mut m: Option<Bytes> = None;
        let mut d: Option<Bytes> = None;
        let mut left = self.mtu - frame::LEN_HEADER - self.skip;
        if let Some(it) = &mut self.meta {
            let msize = it.len();
            if left < msize {
                m = Some(it.split_to(left));
                left = 0;
            } else {
                m = self.meta.take();
                left -= msize;
            }
        }

        if left > 0 {
            if let Some(it) = &mut self.data {
                let dsize = it.len();
                if left < dsize {
                    d = Some(it.split_to(left));
                } else {
                    d = self.data.take();
                }
            }
        }
        self.skip = 0;
        Some(Payload::new(d, m))
    }
}

impl Into<Payload> for Joiner {
    fn into(self) -> Payload {
        let mut bf = BytesMut::new();
        let mut bf2 = BytesMut::new();
        self.inner.into_iter().for_each(|it: Frame| {
            let (d, m) = match it.body {
                Body::RequestResponse(body) => body.split(),
                Body::RequestStream(body) => body.split(),
                Body::RequestChannel(body) => body.split(),
                Body::RequestFNF(body) => body.split(),
                Body::Payload(body) => body.split(),
                _ => (None, None),
            };
            if let Some(raw) = d {
                bf.put(raw);
            }
            if let Some(raw) = m {
                bf2.put(raw);
            }
        });

        let data = if bf.is_empty() {
            None
        } else {
            Some(bf.freeze())
        };
        let metadata = if bf2.is_empty() {
            None
        } else {
            Some(bf2.freeze())
        };
        Payload::new(data, metadata)
    }
}

impl Joiner {
    pub(crate) fn new() -> Joiner {
        Joiner {
            inner: LinkedList::new(),
        }
    }

    pub(crate) fn get_stream_id(&self) -> u32 {
        self.first().get_stream_id()
    }

    pub(crate) fn get_flag(&self) -> u16 {
        self.first().get_flag() & !Frame::FLAG_FOLLOW
    }

    pub(crate) fn first(&self) -> &Frame {
        self.inner.front().expect("No frames pushed!")
    }

    pub(crate) fn push(&mut self, next: Frame) {
        self.inner.push_back(next);
    }
}

#[cfg(test)]
mod tests {

    use bytes::{Buf, Bytes};

    use crate::frame::{self, Frame};
    use crate::payload::Payload;
    use crate::transport::{Joiner, Splitter};

    #[test]
    fn test_joiner() {
        let first = frame::Payload::builder(1, Frame::FLAG_FOLLOW)
            .set_data(Bytes::from("(ROOT)"))
            .set_metadata(Bytes::from("(ROOT)"))
            .build();
        let mut joiner = Joiner::new();
        joiner.push(first);

        for i in 0..10 {
            let flag = if i == 9 { 0u16 } else { Frame::FLAG_FOLLOW };
            let next = frame::Payload::builder(1, flag)
                .set_data(Bytes::from(format!("(data{:04})", i)))
                .set_metadata(Bytes::from(format!("(data{:04})", i)))
                .build();
            joiner.push(next);
        }
        let pa: Payload = joiner.into();
        println!("payload: {:?}", pa);
    }

    #[test]
    fn test_splitter() {
        let input = Payload::builder()
            .set_data_utf8("helloworld")
            .set_metadata_utf8("foobar")
            .build();
        let mut sp = Splitter::new(13);
        for (i, it) in sp.cut(input.clone(), 0).enumerate() {
            println!("{}: {:?}", i, it);
        }
        println!("MODE 100");
        sp = Splitter::new(100);
        for (i, it) in sp.cut(input.clone(), 0).enumerate() {
            println!("{}: {:?}", i, it);
        }
    }
}
