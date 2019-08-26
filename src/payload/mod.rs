extern crate bytes;

use crate::frame;
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct Payload {
  m: Option<Bytes>,
  d: Option<Bytes>,
}

#[derive(Debug, Clone)]
pub struct PayloadBuilder {
  value: Payload,
}

impl PayloadBuilder {
  fn new() -> PayloadBuilder {
    PayloadBuilder {
      value: Payload { m: None, d: None },
    }
  }

  pub fn set_metadata(&mut self, metadata: Bytes) -> &mut PayloadBuilder {
    self.value.m = Some(metadata);
    self
  }
  pub fn set_data(&mut self, data: Bytes) -> &mut PayloadBuilder {
    self.value.d = Some(data);
    self
  }

  pub fn build(&mut self) -> Payload {
    self.value.clone()
  }
}

impl Payload {
  pub fn builder() -> PayloadBuilder {
    PayloadBuilder::new()
  }

  pub fn metadata(&self) -> Option<Bytes> {
    self.m.clone()
  }

  pub fn data(&self) -> Option<Bytes> {
    self.d.clone()
  }
}

impl From<&'static str> for Payload {
  fn from(data: &'static str) -> Payload {
    Payload {
      d: Some(Bytes::from(data)),
      m: None,
    }
  }
}

impl From<(&'static str, &'static str)> for Payload {
  fn from((data, metadata): (&'static str, &'static str)) -> Payload {
    Payload {
      d: Some(Bytes::from(data)),
      m: Some(Bytes::from(metadata)),
    }
  }
}

impl From<&frame::Payload> for Payload {
  fn from(input: &frame::Payload) -> Payload {
    Payload::from((input.get_data(), input.get_metadata()))
  }
}

impl From<(Option<Bytes>, Option<Bytes>)> for Payload {
  fn from((data, metadata): (Option<Bytes>, Option<Bytes>)) -> Payload {
    let mut bu = Payload::builder();
    match metadata {
      Some(b) => {
        bu.set_metadata(b);
        ()
      }
      None => (),
    };
    match data {
      Some(b) => {
        bu.set_data(b);
        ()
      }
      None => (),
    };
    bu.build()
  }
}

impl From<&frame::Setup> for Payload {
  fn from(input: &frame::Setup) -> Payload {
    Payload::from((input.get_data(), input.get_metadata()))
  }
}

impl From<&frame::RequestChannel> for Payload {
  fn from(input: &frame::RequestChannel) -> Payload {
    Payload::from((input.get_data(), input.get_metadata()))
  }
}

impl From<&frame::MetadataPush> for Payload {
  fn from(input: &frame::MetadataPush) -> Payload {
    Payload::from((input.get_data(), input.get_metadata()))
  }
}

impl From<&frame::RequestStream> for Payload {
  fn from(input: &frame::RequestStream) -> Payload {
    Payload::from((input.get_data(), input.get_metadata()))
  }
}

impl From<&frame::RequestFNF> for Payload {
  fn from(input: &frame::RequestFNF) -> Payload {
    Payload::from((input.get_data(), input.get_metadata()))
  }
}

impl From<&frame::RequestResponse> for Payload {
  fn from(input: &frame::RequestResponse) -> Payload {
    Payload::from((input.get_data(), input.get_metadata()))
  }
}
