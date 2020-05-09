use super::misc::{marshal, unmarshal};
use bytes::BytesMut;
use rsocket_rust::error::RSocketError;
use rsocket_rust::extension::{CompositeMetadata, MimeType, RoutingMetadata};
use rsocket_rust::prelude::*;
use rsocket_rust::utils::Writeable;
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;
use std::error::Error;
use std::sync::Arc;

pub struct Requester {
    rsocket: Arc<Box<dyn RSocket>>,
}

pub struct RequestSpec {
    data: Option<Vec<u8>>,
    rsocket: Arc<Box<dyn RSocket>>,
    data_mime_type: MimeType,
    metadatas: LinkedList<(MimeType, Vec<u8>)>,
}

impl From<Box<dyn RSocket>> for Requester {
    fn from(rsocket: Box<dyn RSocket>) -> Requester {
        Requester {
            rsocket: Arc::new(rsocket),
        }
    }
}

impl Requester {
    pub fn route(&self, route: &str) -> RequestSpec {
        let routing = RoutingMetadata::builder().push_str(route).build();
        let mut buf = BytesMut::new();
        routing.write_to(&mut buf);

        let mut metadatas: LinkedList<(MimeType, Vec<u8>)> = Default::default();
        metadatas.push_back((MimeType::MESSAGE_X_RSOCKET_ROUTING_V0, buf.to_vec()));
        RequestSpec {
            data: None,
            rsocket: self.rsocket.clone(),
            data_mime_type: MimeType::APPLICATION_JSON,
            metadatas,
        }
    }
}

impl RequestSpec {
    pub fn metadata<T, M>(&mut self, metadata: &T, mime_type: M) -> Result<(), Box<dyn Error>>
    where
        T: Sized + Serialize,
        M: Into<MimeType>,
    {
        let mime_type = mime_type.into();
        let mut b = BytesMut::new();
        marshal(&mime_type, &mut b, metadata)?;
        self.metadatas.push_back((mime_type, b.to_vec()));
        Ok(())
    }

    pub fn metadata_raw<I, M>(&mut self, metadata: I, mime_type: M) -> Result<(), Box<dyn Error>>
    where
        I: Into<Vec<u8>>,
        M: Into<MimeType>,
    {
        self.metadatas
            .push_back((mime_type.into(), metadata.into()));
        Ok(())
    }

    pub fn data<T>(&mut self, data: &T) -> Result<(), Box<dyn Error>>
    where
        T: Sized + Serialize,
    {
        let mut bf = BytesMut::new();
        marshal(&self.data_mime_type, &mut bf, data)?;
        self.data = Some(bf.to_vec());
        Ok(())
    }

    pub fn data_raw<I>(&mut self, data: I) -> Result<(), Box<dyn Error>>
    where
        I: Into<Vec<u8>>,
    {
        self.data = Some(data.into());
        Ok(())
    }

    pub async fn retrieve_mono(self) -> Unpacker {
        let (req, mime_type, rsocket) = self.preflight();
        let res = rsocket.request_response(req).await;
        Unpacker {
            mime_type,
            inner: res,
        }
    }

    #[inline]
    fn preflight(self) -> (Payload, MimeType, Arc<Box<dyn RSocket>>) {
        let mut b = BytesMut::new();
        let mut c = CompositeMetadata::builder();
        for (mime_type, raw) in self.metadatas.into_iter() {
            c = c.push(mime_type, raw);
        }
        c.build().write_to(&mut b);

        let mut bu = Payload::builder().set_metadata(b.to_vec());
        if let Some(raw) = self.data {
            bu = bu.set_data(raw);
        }
        (bu.build(), self.data_mime_type, self.rsocket)
    }
}

pub struct Unpacker {
    mime_type: MimeType,
    inner: Result<Payload, RSocketError>,
}

impl Unpacker {
    pub fn block<'a, T>(&'a self) -> Result<Option<T>, Box<dyn Error>>
    where
        T: Deserialize<'a>,
    {
        match &self.inner {
            Ok(inner) => match inner.data() {
                Some(raw) => Ok(Some(unmarshal(&self.mime_type, &raw.as_ref())?)),
                None => Ok(None),
            },
            Err(e) => Err(format!("{}", e).into()),
        }
    }
}
