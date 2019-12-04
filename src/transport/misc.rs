use crate::errors::{ErrorKind, RSocketError};
use crate::frame::{self};
use crate::payload::{Payload, SetupPayload};
use crate::result::RSocketResult;
use crate::spi::RSocket;
use futures::future;

use std::{
    collections::HashMap,
    future::Future,
    sync::{
        atomic::{AtomicI64, AtomicU32, Ordering},
        Arc, Mutex, RwLock,
    },
};
use tokio::sync::{mpsc, oneshot};

#[derive(Debug, Clone)]
pub(crate) struct StreamID {
    inner: Arc<AtomicU32>,
}

impl StreamID {
    pub(crate) fn new(value: u32) -> StreamID {
        let inner = Arc::new(AtomicU32::new(value));
        StreamID { inner }
    }

    pub(crate) fn next(&self) -> u32 {
        let counter = self.inner.clone();
        counter.fetch_add(2, Ordering::SeqCst)
    }
}

impl From<u32> for StreamID {
    fn from(v: u32) -> StreamID {
        StreamID::new(v)
    }
}

#[derive(Debug, Clone)]
pub(crate) struct Counter {
    inner: Arc<AtomicI64>,
}

impl Counter {
    pub(crate) fn new(value: i64) -> Counter {
        Counter {
            inner: Arc::new(AtomicI64::new(value)),
        }
    }

    pub(crate) fn count_down(&self) -> i64 {
        let c = self.inner.clone();
        c.fetch_add(-1, Ordering::SeqCst)
    }
}

#[inline]
pub(crate) fn debug_frame(snd: bool, f: &frame::Frame) {
    if snd {
        debug!("===> SND: {:?}", f);
    } else {
        debug!("<=== RCV: {:?}", f);
    }
}
