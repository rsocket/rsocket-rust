extern crate bytes;

use bytes::Bytes;

#[derive(Debug,Clone)]
pub struct Payload {
  m: Option<Bytes>,
  d: Option<Bytes>,
}

#[derive(Debug,Clone)]
pub struct PayloadBuilder{
  value: Payload
}

impl PayloadBuilder{
  fn new() -> PayloadBuilder{
    PayloadBuilder{
      value: Payload{
        m: None,
        d: None,
      }
    }
  }

  pub fn set_metadata(&mut self,metadata: Bytes) -> &mut PayloadBuilder{
    self.value.m = Some(metadata);
    self
  }
  pub fn set_data(&mut self,data: Bytes) -> &mut PayloadBuilder{
    self.value.d = Some(data);
    self
  }

  pub fn build(&mut self) -> Payload{
    self.value.clone()
  }
}

impl Payload {

  pub fn builder() -> PayloadBuilder{
    PayloadBuilder::new()
  }


  pub fn metadata(&self) -> Option<Bytes> {
    self.m.clone()
  }

  pub fn data(&self) -> Option<Bytes> {
    self.d.clone()
  }
}
