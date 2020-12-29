use super::misc::bytes_to_utf8;
use crate::frame::Setup;
use crate::utils::DEFAULT_MIME_TYPE;
use bytes::Bytes;
use std::time::Duration;

#[derive(Debug)]
pub struct SetupPayload {
    m: Option<Bytes>,
    d: Option<Bytes>,
    keepalive: (Duration, Duration),
    mime_m: Option<Bytes>,
    mime_d: Option<Bytes>,
}

#[derive(Debug)]
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
                mime_m: Some(Bytes::from(DEFAULT_MIME_TYPE.to_owned())),
                mime_d: Some(Bytes::from(DEFAULT_MIME_TYPE.to_owned())),
            },
        }
    }

    pub fn set_metadata<A>(mut self, metadata: A) -> Self
    where
        A: Into<Vec<u8>>,
    {
        self.inner.m = Some(Bytes::from(metadata.into()));
        self
    }

    pub fn set_metadata_utf8(mut self, metadata: &str) -> Self {
        self.inner.m = Some(Bytes::from(String::from(metadata)));
        self
    }

    pub(crate) fn set_data_bytes(mut self, data: Option<Bytes>) -> Self {
        self.inner.d = data;
        self
    }

    pub(crate) fn set_metadata_bytes(mut self, data: Option<Bytes>) -> Self {
        self.inner.m = data;
        self
    }

    pub fn set_data<A>(mut self, data: A) -> Self
    where
        A: Into<Vec<u8>>,
    {
        self.inner.d = Some(Bytes::from(data.into()));
        self
    }

    pub fn set_data_utf8(mut self, data: &str) -> Self {
        self.inner.d = Some(Bytes::from(data.to_owned()));
        self
    }

    pub fn set_keepalive(
        mut self,
        tick_period: Duration,
        ack_timeout: Duration,
        missed_acks: u64,
    ) -> Self {
        let lifetime_mills = (ack_timeout.as_millis() as u64) * missed_acks;
        self.inner.keepalive = (tick_period, Duration::from_millis(lifetime_mills));
        self
    }

    pub fn set_data_mime_type(mut self, mime: &str) -> Self {
        self.inner.mime_d = Some(Bytes::from(mime.to_owned()));
        self
    }
    pub fn set_metadata_mime_type(mut self, mime: &str) -> Self {
        self.inner.mime_m = Some(Bytes::from(mime.to_owned()));
        self
    }

    pub fn build(self) -> SetupPayload {
        self.inner
    }
}

impl SetupPayload {
    pub fn metadata(&self) -> Option<&Bytes> {
        self.m.as_ref()
    }

    pub fn data(&self) -> Option<&Bytes> {
        self.d.as_ref()
    }

    pub fn split(self) -> (Option<Bytes>, Option<Bytes>) {
        (self.d, self.m)
    }

    pub fn keepalive_interval(&self) -> Duration {
        self.keepalive.0
    }

    pub fn keepalive_lifetime(&self) -> Duration {
        self.keepalive.1
    }

    pub fn metadata_mime_type(&self) -> Option<&str> {
        bytes_to_utf8(&self.mime_m)
    }

    pub fn data_mime_type(&self) -> Option<&str> {
        bytes_to_utf8(&self.mime_d)
    }
}

impl From<Setup> for SetupPayload {
    fn from(input: Setup) -> SetupPayload {
        let mut bu = SetupPayload::builder();
        // TODO: fill other properties.
        if let Some(m) = input.get_mime_data() {
            bu = bu.set_data_mime_type(m);
        }
        if let Some(m) = input.get_mime_metadata() {
            bu = bu.set_metadata_mime_type(m);
        }
        let keepalive = (input.get_keepalive(), input.get_lifetime());
        let (d, m) = input.split();
        bu.inner.d = d;
        bu.inner.m = m;
        let mut pa = bu.build();
        pa.keepalive = keepalive;
        pa
    }
}
