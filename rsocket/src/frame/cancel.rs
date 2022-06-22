use super::{Body, Frame};

#[derive(Debug, Eq, PartialEq)]
pub struct Cancel {}

pub struct CancelBuilder {
    stream_id: u32,
    flag: u16,
}

impl CancelBuilder {
    pub fn build(self) -> Frame {
        Frame::new(self.stream_id, Body::Cancel(), self.flag)
    }
}

impl Cancel {
    pub fn builder(stream_id: u32, flag: u16) -> CancelBuilder {
        CancelBuilder { stream_id, flag }
    }
}
