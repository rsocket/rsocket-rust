use super::misc::{marshal, unmarshal};
use bytes::BytesMut;
use rsocket_rust::error::RSocketError;
use rsocket_rust::extension::{
    CompositeMetadata, CompositeMetadataEntry, MimeType, RoutingMetadata,
};
use rsocket_rust::prelude::*;
use rsocket_rust::utils::Writeable;
use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;
use serde::{Deserialize, Serialize};
use std::collections::LinkedList;
use std::error::Error;
use std::net::SocketAddr;
use std::result::Result;
use std::sync::Arc;
use url::Url;

enum TransportKind {
    TCP(String, u16),
    WS(String),
}

pub struct Requester {
    rsocket: Arc<Box<dyn RSocket>>,
}

pub struct RequestSpec {
    data: Option<Vec<u8>>,
    rsocket: Arc<Box<dyn RSocket>>,
    data_mime_type: MimeType,
    metadatas: LinkedList<(MimeType, Vec<u8>)>,
}

pub struct RequesterBuilder {
    data_mime_type: Option<MimeType>,
    route: Option<String>,
    metadata: Vec<CompositeMetadataEntry>,
    data: Option<Vec<u8>>,
    tp: Option<TransportKind>,
}

impl Default for RequesterBuilder {
    fn default() -> Self {
        Self {
            data_mime_type: None,
            route: None,
            metadata: Default::default(),
            data: None,
            tp: None,
        }
    }
}

impl RequesterBuilder {
    pub fn data_mime_type<I>(mut self, mime_type: I) -> Self
    where
        I: Into<MimeType>,
    {
        self.data_mime_type = Some(mime_type.into());
        self
    }

    pub fn setup_route<I>(mut self, route: I) -> Self
    where
        I: Into<String>,
    {
        self.route = Some(route.into());
        self
    }

    pub fn setup_data<D>(mut self, data: &D) -> Self
    where
        D: Sized + Serialize,
    {
        // TODO: lazy set
        let mut bf = BytesMut::new();
        let result = match &self.data_mime_type {
            Some(m) => marshal(m, &mut bf, data),
            None => marshal(&MimeType::APPLICATION_JSON, &mut bf, data),
        };
        match result {
            Ok(()) => {
                self.data = Some(bf.to_vec());
            }
            Err(e) => {
                error!("marshal failed: {:?}", e);
            }
        }
        self
    }

    pub fn setup_metadata<M, T>(mut self, metadata: &M, mime_type: T) -> Self
    where
        M: Sized + Serialize,
        T: Into<MimeType>,
    {
        // TODO: lazy set
        let mut bf = BytesMut::new();
        let mime_type = mime_type.into();
        match marshal(&mime_type, &mut bf, metadata) {
            Ok(()) => {
                let entry = CompositeMetadataEntry::new(mime_type, bf.freeze());
                self.metadata.push(entry);
            }
            Err(e) => error!("marshal failed: {:?}", e),
        }
        self
    }

    pub fn connect_tcp<A>(mut self, host: A, port: u16) -> Self
    where
        A: Into<String>,
    {
        self.tp = Some(TransportKind::TCP(host.into(), port));
        self
    }

    pub fn connect_websocket<I>(mut self, url: I) -> Self
    where
        I: Into<String>,
    {
        self.tp = Some(TransportKind::WS(url.into()));
        self
    }

    pub async fn build(self) -> Result<Requester, Box<dyn Error + Send + Sync>> {
        let data_mime_type = self.data_mime_type.unwrap_or(MimeType::APPLICATION_JSON);

        let mut added = 0usize;
        let mut composite_builder = CompositeMetadata::builder();

        if let Some(s) = self.route {
            let routing = RoutingMetadata::builder().push(s).build();
            composite_builder =
                composite_builder.push(MimeType::MESSAGE_X_RSOCKET_ROUTING_V0, routing.bytes());
            added += 1;
        }
        for it in self.metadata.into_iter() {
            composite_builder = composite_builder.push_entry(it);
            added += 1;
        }

        let mut payload_builder = Payload::builder();

        if added > 0 {
            payload_builder = payload_builder.set_metadata(composite_builder.build());
        }

        if let Some(raw) = self.data {
            payload_builder = payload_builder.set_data(raw);
        }

        let setup = payload_builder.build();

        match self.tp {
            Some(TransportKind::TCP(h, p)) => {
                let addr: SocketAddr = format!("{}:{}", h, p).parse()?;
                let cli = RSocketFactory::connect()
                    .data_mime_type(data_mime_type.as_ref())
                    .setup(setup)
                    .metadata_mime_type(MimeType::MESSAGE_X_RSOCKET_COMPOSITE_METADATA_V0.as_ref())
                    .transport(TcpClientTransport::from(addr))
                    .start()
                    .await?;
                let rsocket: Box<dyn RSocket> = Box::new(cli);
                let requester = Requester::from(rsocket);
                Ok(requester)
            }
            Some(TransportKind::WS(u)) => {
                let url = Url::parse(&u)?;
                let cli = RSocketFactory::connect()
                    .data_mime_type(data_mime_type.as_ref())
                    .setup(setup)
                    .metadata_mime_type(MimeType::MESSAGE_X_RSOCKET_COMPOSITE_METADATA_V0.as_ref())
                    .transport(WebsocketClientTransport::from(url))
                    .start()
                    .await?;
                let rsocket: Box<dyn RSocket> = Box::new(cli);
                let requester = Requester::from(rsocket);
                Ok(requester)
            }
            None => Err("Missing transport!".into()),
        }
    }
}

impl From<Box<dyn RSocket>> for Requester {
    fn from(rsocket: Box<dyn RSocket>) -> Requester {
        Requester {
            rsocket: Arc::new(rsocket),
        }
    }
}

impl Requester {
    pub fn builder() -> RequesterBuilder {
        RequesterBuilder::default()
    }

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
