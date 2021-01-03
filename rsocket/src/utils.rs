use super::spi::{Flux, RSocket};
use crate::error::RSocketError;
use crate::payload::Payload;
use crate::runtime;
use crate::Result;
use async_stream::stream;
use async_trait::async_trait;
use bytes::{Buf, BufMut, Bytes, BytesMut};
use futures::{pin_mut, FutureExt, Sink, SinkExt, Stream, StreamExt};
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use tokio::sync::mpsc;

pub const DEFAULT_MIME_TYPE: &str = "application/binary";

pub struct EchoRSocket;

#[async_trait]
impl RSocket for EchoRSocket {
    async fn metadata_push(&self, req: Payload) -> Result<()> {
        info!("{:?}", req);
        Ok(())
    }

    async fn fire_and_forget(&self, req: Payload) -> Result<()> {
        info!("{:?}", req);
        Ok(())
    }

    async fn request_response(&self, req: Payload) -> Result<Option<Payload>> {
        info!("{:?}", req);
        Ok(Some(req))
    }

    fn request_stream(&self, req: Payload) -> Flux<Result<Payload>> {
        info!("{:?}", req);
        Box::pin(stream! {
            yield Ok(req);
        })
    }

    fn request_channel(&self, mut reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        let (sender, receiver) = mpsc::unbounded_channel();
        runtime::spawn(async move {
            while let Some(it) = reqs.next().await {
                info!("{:?}", it);
                sender.send(it).unwrap();
            }
        });

        Box::pin(receiver)
        // or returns directly
        // reqs
    }
}

pub(crate) struct EmptyRSocket;

#[async_trait]
impl RSocket for EmptyRSocket {
    async fn metadata_push(&self, _req: Payload) -> Result<()> {
        Err(RSocketError::ApplicationException("UNIMPLEMENT".into()).into())
    }

    async fn fire_and_forget(&self, _req: Payload) -> Result<()> {
        Err(RSocketError::ApplicationException("UNIMPLEMENT".into()).into())
    }

    async fn request_response(&self, _req: Payload) -> Result<Option<Payload>> {
        Err(RSocketError::ApplicationException("UNIMPLEMENT".into()).into())
    }

    fn request_stream(&self, _req: Payload) -> Flux<Result<Payload>> {
        Box::pin(futures::stream::empty())
    }

    fn request_channel(&self, _reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        Box::pin(futures::stream::empty())
    }
}

pub trait Writeable {
    fn write_to(&self, bf: &mut BytesMut);
    fn len(&self) -> usize;
    fn is_empty(&self) -> bool {
        self.len() == 0
    }
    fn bytes(&self) -> Vec<u8> {
        let mut b = BytesMut::new();
        self.write_to(&mut b);
        b.to_vec()
    }
}

#[allow(non_camel_case_types)]
#[derive(Default, Clone, Copy, Debug)]
pub struct u24(u32);

macro_rules! ux {
    ($type:ident) => {
        impl From<$type> for u24 {
            fn from(n: $type) -> Self {
                assert!(n <= Self::MAX as $type);
                Self(n as u32)
            }
        }
        impl Into<$type> for u24 {
            fn into(self) -> $type {
                if (std::$type::MAX as u64) < (std::u16::MAX as u64) {
                    assert!(self.0 <= (std::$type::MAX as u32));
                }
                self.0 as $type
            }
        }
    };
}

macro_rules! ix {
    ($type:ident) => {
        impl From<$type> for u24 {
            fn from(n: $type) -> Self {
                assert!(n >= Self::MIN as $type && n <= Self::MAX as $type);
                Self(n as u32)
            }
        }
        impl Into<$type> for u24 {
            fn into(self) -> $type {
                if (std::$type::MAX as u64) < (std::u16::MAX as u64) {
                    assert!(self.0 <= (std::$type::MAX as u32));
                }
                self.0 as $type
            }
        }
    };
}

ux!(u8);
ux!(u16);
ux!(u32);
ux!(u64);
ux!(usize);
ix!(i8);
ix!(i16);
ix!(i32);
ix!(i64);
ix!(isize);

impl Writeable for u24 {
    fn write_to(&self, bf: &mut BytesMut) {
        bf.put_u8((0xFF & (self.0 >> 16)) as u8);
        bf.put_u8((0xFF & (self.0 >> 8)) as u8);
        bf.put_u8((0xFF & self.0) as u8);
    }
    fn len(&self) -> usize {
        3
    }
}

impl u24 {
    pub const MAX: u32 = 0x00FF_FFFF;
    pub const MIN: u32 = 0;

    pub fn parse(b: &[u8]) -> u24 {
        assert!(b.len() > 2);
        let mut n = 0u32;
        n += (b[0] as u32) << 16;
        n += (b[1] as u32) << 8;
        n += b[2] as u32;
        u24(n)
    }

    pub fn read(bf: &mut BytesMut) -> u24 {
        Self::parse(bf)
    }

    pub fn read_advance(bf: &mut BytesMut) -> u24 {
        let raw = bf.split_to(3);
        Self::parse(&raw)
    }
}
