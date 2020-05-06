use super::misc::{marshal, unmarshal};
use bytes::{BufMut, BytesMut};
use rsocket_rust::error::RSocketError;
use rsocket_rust::extension::{
    CompositeMetadata, MimeType, RoutingMetadata, MIME_APPLICATION_JSON,
    MIME_MESSAGE_X_RSOCKET_ROUTING_V0,
};
use rsocket_rust::prelude::*;
use rsocket_rust::utils::Writeable;
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;
use std::error::Error;

pub struct RequesterBuilder {
    data_mime_type: MimeType,
    route: Option<String>,
    data: Option<Vec<u8>>,
}

impl Default for RequesterBuilder {
    fn default() -> Self {
        Self {
            data_mime_type: MIME_APPLICATION_JSON,
            route: None,
            data: None,
        }
    }
}

impl RequesterBuilder {
    pub fn data_mime_type<I>(mut self, mime_type: I) -> Self
    where
        I: Into<MimeType>,
    {
        self.data_mime_type = mime_type.into();
        self
    }

    pub fn setup_route<I>(mut self, route: I) -> Self
    where
        I: Into<String>,
    {
        self.route = Some(route.into());
        self
    }

    pub fn setup_data<D>(mut self, data: D) -> Self
    where
        D: Into<Vec<u8>>,
    {
        self.data = Some(data.into());
        self
    }

    pub fn build<S>(self) -> Requester<S>
    where
        S: RSocket + Clone,
    {
        todo!("build requester")
    }
}

pub struct Requester<S>
where
    S: RSocket + Clone,
{
    rsocket: S,
}

pub struct RequestSpec<S>
where
    S: RSocket + Clone,
{
    data: Option<Vec<u8>>,
    rsocket: S,
    data_mime_type: MimeType,
    metadatas: LinkedList<(MimeType, Vec<u8>)>,
}

impl<S> From<S> for Requester<S>
where
    S: RSocket + Clone,
{
    fn from(rsocket: S) -> Requester<S> {
        Requester { rsocket }
    }
}

impl<C> Requester<C>
where
    C: RSocket + Clone,
{
    pub fn route(&self, route: &str) -> RequestSpec<C> {
        let routing = RoutingMetadata::builder().push_str(route).build();
        let mut buf = BytesMut::new();
        routing.write_to(&mut buf);

        let mut metadatas: LinkedList<(MimeType, Vec<u8>)> = Default::default();
        metadatas.push_back((MIME_MESSAGE_X_RSOCKET_ROUTING_V0, buf.to_vec()));
        RequestSpec {
            data: None,
            rsocket: self.rsocket.clone(),
            data_mime_type: MIME_APPLICATION_JSON,
            metadatas,
        }
    }
}

impl<C> RequestSpec<C>
where
    C: RSocket + Clone,
{
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
    fn preflight(self) -> (Payload, MimeType, C) {
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
