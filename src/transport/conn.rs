use crate::frame::Frame;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

pub struct Conn {
  handlers: Mutex<HashMap<u16, fn(&Frame)>>,
}

impl Conn {
  pub fn new() -> Conn {
    let mut m = HashMap::new();
    Conn {
      handlers: Mutex::new(m),
    }
  }

  pub fn bingo(&self, f: &Frame) {
    let mut h = self.handlers.lock().unwrap();
    match h.get(&f.get_frame_type()) {
      Some(call) => call(f),
      None => (),
    }
  }

  pub fn register(&self, frame_type: u16, handler: fn(&Frame)) {
    let mut db = self.handlers.lock().unwrap();
    db.insert(frame_type, handler);
  }
}
