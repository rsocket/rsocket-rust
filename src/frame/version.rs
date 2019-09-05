extern crate bytes;

use bytes::{BufMut, BytesMut};

#[derive(Debug, Copy, Clone)]
pub struct Version {
  major: u16,
  minor: u16,
}

impl Version {
  pub fn default() -> Version {
    Version { major: 1, minor: 0 }
  }

  pub fn new(major: u16, minor: u16) -> Version {
    Version { major, minor }
  }

  pub fn get_major(&self) -> u16 {
    self.major
  }

  pub fn get_minor(&self) -> u16 {
    self.minor
  }

  pub fn write_to(&self, bf: &mut BytesMut) {
    bf.put_u16_be(self.major);
    bf.put_u16_be(self.minor);
  }
}
