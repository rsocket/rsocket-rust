use bytes::{Buf, BufMut, Bytes, BytesMut};

use crate::utils::Writeable;

#[derive(Debug, Copy, Clone, Eq, PartialEq)]
pub struct Version {
    major: u16,
    minor: u16,
}

impl Default for Version {
    fn default() -> Version {
        Version { major: 1, minor: 0 }
    }
}

impl Writeable for Version {
    fn write_to(&self, bf: &mut BytesMut) {
        bf.put_u16(self.major);
        bf.put_u16(self.minor);
    }
    fn len(&self) -> usize {
        4
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
}
