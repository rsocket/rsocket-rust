use crate::frame;
use bytes::Bytes;

#[derive(Debug, Clone)]
pub struct Payload {
    m: Option<Bytes>,
    d: Option<Bytes>,
}

#[derive(Debug)]
pub struct PayloadBuilder {
    value: Payload,
}

impl PayloadBuilder {
    fn new() -> PayloadBuilder {
        PayloadBuilder {
            value: Payload { m: None, d: None },
        }
    }

    pub fn set_metadata(mut self, metadata: Bytes) -> Self {
        self.value.m = Some(metadata);
        self
    }

    pub fn set_metadata_utf8(self, metadata: &str) -> Self {
        self.set_metadata(Bytes::from(String::from(metadata)))
    }

    pub fn set_data(mut self, data: Bytes) -> Self {
        self.value.d = Some(data);
        self
    }

    pub fn set_data_utf8(self, data: &str) -> Self {
        self.set_data(Bytes::from(String::from(data)))
    }

    pub fn build(self) -> Payload {
        self.value
    }
}

impl Payload {
    pub fn builder() -> PayloadBuilder {
        PayloadBuilder::new()
    }

    pub fn metadata(&self) -> &Option<Bytes> {
        &self.m
    }

    pub fn data(&self) -> &Option<Bytes> {
        &self.d
    }

    pub fn split(self) -> (Option<Bytes>, Option<Bytes>) {
        (self.d, self.m)
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

impl From<(Option<Bytes>, Option<Bytes>)> for Payload {
    fn from((data, metadata): (Option<Bytes>, Option<Bytes>)) -> Payload {
        let mut bu = Payload::builder();
        if let Some(b) = metadata {
            bu = bu.set_metadata(b);
        }
        if let Some(b) = data {
            bu = bu.set_data(b);
        }
        bu.build()
    }
}

impl From<frame::Payload> for Payload {
    fn from(input: frame::Payload) -> Payload {
        let d = input.get_data().clone();
        let m = input.get_metadata().clone();
        Payload::from((d, m))
    }
}

impl From<frame::Setup> for Payload {
    fn from(input: frame::Setup) -> Payload {
        Payload::from(input.split())
    }
}

impl From<frame::RequestChannel> for Payload {
    fn from(input: frame::RequestChannel) -> Payload {
        Payload::from(input.split())
    }
}

impl From<frame::MetadataPush> for Payload {
    fn from(input: frame::MetadataPush) -> Payload {
        Payload::from(input.split())
    }
}

impl From<frame::RequestStream> for Payload {
    fn from(input: frame::RequestStream) -> Payload {
        Payload::from(input.split())
    }
}

impl From<frame::RequestFNF> for Payload {
    fn from(input: frame::RequestFNF) -> Payload {
        Payload::from(input.split())
    }
}

impl From<frame::RequestResponse> for Payload {
    fn from(input: frame::RequestResponse) -> Payload {
        Payload::from(input.split())
    }
}
