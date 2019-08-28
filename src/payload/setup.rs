extern crate bytes;

use crate::mime::MIME_BINARY;
use bytes::Bytes;
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct SetupPayload {
  m: Option<Bytes>,
  d: Option<Bytes>,
  keepalive: (Duration, Duration),
  mime_m: Option<String>,
  mime_d: Option<String>,
}

pub struct SetupPayloadBuilder {
  inner: SetupPayload,
}

impl SetupPayload {
  pub fn builder() -> SetupPayloadBuilder {
    SetupPayloadBuilder::new()
  }
}

impl SetupPayloadBuilder {
  fn new() -> SetupPayloadBuilder {
    SetupPayloadBuilder {
      inner: SetupPayload {
        m: None,
        d: None,
        keepalive: (Duration::from_secs(20), Duration::from_secs(90)),
        mime_m: Some(String::from(MIME_BINARY)),
        mime_d: Some(String::from(MIME_BINARY)),
      },
    }
  }

  pub fn set_metadata(&mut self, metadata: Bytes) -> &mut SetupPayloadBuilder {
    self.inner.m = Some(metadata);
    self
  }

  pub fn set_data(&mut self, data: Bytes) -> &mut SetupPayloadBuilder {
    self.inner.d = Some(data);
    self
  }

  pub fn set_keepalive(
    &mut self,
    tick_period: Duration,
    ack_timeout: Duration,
    missed_acks: u64,
  ) -> &mut SetupPayloadBuilder {
    let lifetime_mills = (ack_timeout.as_millis() as u64) * missed_acks;
    self.inner.keepalive = (tick_period, Duration::from_millis(lifetime_mills));
    self
  }

  pub fn set_data_mime_type(&mut self, mime: String) -> &mut SetupPayloadBuilder {
    self.inner.mime_d = Some(mime);
    self
  }
  pub fn set_metadata_mime_type(&mut self, mime: String) -> &mut SetupPayloadBuilder {
    self.inner.mime_m = Some(mime);
    self
  }

  pub fn build(&mut self) -> SetupPayload {
    self.inner.clone()
  }
}

impl SetupPayload {
  pub fn metadata(&self) -> Option<Bytes> {
    self.m.clone()
  }

  pub fn data(&self) -> Option<Bytes> {
    self.d.clone()
  }

  pub fn keepalive_interval(&self) -> Duration {
    self.keepalive.0.clone()
  }

  pub fn keepalive_lifetime(&self) -> Duration {
    self.keepalive.1.clone()
  }

  pub fn metadata_mime_type(&self) -> Option<String> {
    self.mime_m.clone()
  }

  pub fn data_mime_type(&self) -> Option<String> {
    self.mime_d.clone()
  }
}
