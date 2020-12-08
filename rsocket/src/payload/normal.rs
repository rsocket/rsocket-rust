use super::misc::bytes_to_utf8;
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

    pub fn set_data<A>(mut self, data: A) -> Self
    where
        A: Into<Vec<u8>>,
    {
        self.value.d = Some(Bytes::from(data.into()));
        self
    }

    pub fn set_metadata<A>(mut self, metadata: A) -> Self
    where
        A: Into<Vec<u8>>,
    {
        self.value.m = Some(Bytes::from(metadata.into()));
        self
    }

    pub fn set_metadata_utf8(mut self, metadata: &str) -> Self {
        self.value.m = Some(Bytes::from(metadata.to_owned()));
        self
    }

    pub fn set_data_utf8(mut self, data: &str) -> Self {
        self.value.d = Some(Bytes::from(data.to_owned()));
        self
    }

    pub fn build(self) -> Payload {
        self.value
    }
}

impl Payload {
    pub fn new(data: Option<Bytes>, metadata: Option<Bytes>) -> Payload {
        Payload {
            d: data,
            m: metadata,
        }
    }

    pub fn builder() -> PayloadBuilder {
        PayloadBuilder::new()
    }

    pub fn metadata(&self) -> Option<&Bytes> {
        match &self.m {
            Some(b) => Some(b),
            None => None,
        }
    }

    pub fn data(&self) -> Option<&Bytes> {
        match &self.d {
            Some(b) => Some(b),
            None => None,
        }
    }

    pub fn data_utf8(&self) -> Option<&str> {
        bytes_to_utf8(&self.d)
    }

    pub fn metadata_utf8(&self) -> Option<&str> {
        bytes_to_utf8(&self.m)
    }

    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn len(&self) -> usize {
        let mut n = 0;
        if let Some(it) = &self.m {
            n += it.len();
        }
        if let Some(it) = &self.d {
            n += it.len();
        }
        n
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

impl From<frame::Payload> for Payload {
    fn from(input: frame::Payload) -> Payload {
        let (d, m) = input.split();
        Payload::new(d, m)
    }
}

impl From<frame::Setup> for Payload {
    fn from(input: frame::Setup) -> Payload {
        let (d, m) = input.split();
        Payload::new(d, m)
    }
}

impl From<frame::RequestChannel> for Payload {
    fn from(input: frame::RequestChannel) -> Payload {
        let (d, m) = input.split();
        Payload::new(d, m)
    }
}

impl From<frame::MetadataPush> for Payload {
    fn from(input: frame::MetadataPush) -> Payload {
        let (d, m) = input.split();
        Payload::new(d, m)
    }
}

impl From<frame::RequestStream> for Payload {
    fn from(input: frame::RequestStream) -> Payload {
        let (d, m) = input.split();
        Payload::new(d, m)
    }
}

impl From<frame::RequestFNF> for Payload {
    fn from(input: frame::RequestFNF) -> Payload {
        let (d, m) = input.split();
        Payload::new(d, m)
    }
}

impl From<frame::RequestResponse> for Payload {
    fn from(input: frame::RequestResponse) -> Payload {
        let (d, m) = input.split();
        Payload::new(d, m)
    }
}
