use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

#[derive(Debug, Clone)]
pub struct StreamID {
  inner: Arc<AtomicU32>,
}

impl StreamID {
  fn new(value: u32) -> StreamID {
    let inner = Arc::new(AtomicU32::new(value));
    StreamID { inner: inner }
  }

  pub fn next(&self) -> u32 {
    let counter = self.inner.clone();
    counter.fetch_add(2, Ordering::SeqCst)
  }
}

impl From<u32> for StreamID {
  fn from(v: u32) -> StreamID {
    StreamID::new(v)
  }
}
