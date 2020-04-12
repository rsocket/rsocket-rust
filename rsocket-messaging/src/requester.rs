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
    data_buf: BytesMut,
    rsocket: S,
    data_mime_type: MimeType,
    metadatas: LinkedList<(MimeType, Vec<u8>)>,
}

impl<C> Requester<C>
where
    C: RSocket + Clone,
{
    pub fn new(rsocket: C) -> Requester<C> {
        Requester { rsocket }
    }

    pub fn route(&self, route: &str) -> RequestSpec<C> {
        let routing = RoutingMetadata::builder().push_str(route).build();
        let mut buf = BytesMut::new();
        routing.write_to(&mut buf);

        let mut metadatas: LinkedList<(MimeType, Vec<u8>)> = Default::default();
        metadatas.push_back((MIME_MESSAGE_X_RSOCKET_ROUTING_V0, buf.to_vec()));
        RequestSpec {
            data_buf: BytesMut::new(),
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
    pub fn metadata<T>(&mut self, metadata: &T, mime_type: &str) -> Result<(), Box<dyn Error>>
    where
        T: Sized + Serialize,
    {
        let mime_type = MimeType::from(mime_type);
        let mut b = BytesMut::new();
        marshal(&mime_type, &mut b, metadata)?;
        self.metadatas.push_back((mime_type, b.to_vec()));
        Ok(())
    }

    pub fn data<T>(&mut self, data: &T) -> Result<(), Box<dyn Error>>
    where
        T: Sized + Serialize,
    {
        marshal(&self.data_mime_type, &mut self.data_buf, data)
    }

    pub async fn retrieve_mono(&self) -> Unpacker {
        let req = self.to_req();
        let res = self.rsocket.request_response(req).await;
        Unpacker {
            mime_type: self.data_mime_type.clone(),
            inner: res,
        }
    }

    fn to_req(&self) -> Payload {
        let mut b = BytesMut::new();
        let mut c = CompositeMetadata::builder();
        for (a, b) in self.metadatas.iter() {
            c = c.push(a.clone(), b);
        }
        c.build().write_to(&mut b);
        Payload::builder()
            .set_metadata(b.to_vec())
            .set_data(self.data_buf.to_vec())
            .build()
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
