use bytes::{Buf, BufMut, Bytes, BytesMut};

#[derive(Debug, Copy, Clone, PartialEq)]
pub struct Version {
  major: u16,
  minor: u16,
}

impl Default for Version {
  fn default() -> Version {
    Version { major: 1, minor: 0 }
  }
}

impl Version {
  pub fn new(major: u16, minor: u16) -> Version {
    Version { major, minor }
  }

  pub fn get_major(self) -> u16 {
    self.major
  }

  pub fn get_minor(self) -> u16 {
    self.minor
  }

  pub fn write_to(self, bf: &mut BytesMut) {
    bf.put_u16(self.major);
    bf.put_u16(self.minor);
  }
}
