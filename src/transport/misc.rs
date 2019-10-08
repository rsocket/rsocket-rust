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
        atomic::{AtomicU32, Ordering},
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

#[derive(Debug)]
pub(crate) enum Handler {
    Request(oneshot::Sender<Payload>),
    Stream(mpsc::Sender<Payload>),
}

#[derive(Debug)]
pub(crate) struct Handlers {
    map: RwLock<HashMap<u32, Handler>>,
}

impl Handlers {
    pub(crate) fn new() -> Handlers {
        Handlers {
            map: RwLock::new(HashMap::new()),
        }
    }
}
