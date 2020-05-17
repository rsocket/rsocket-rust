use super::misc::{self, marshal, unmarshal};
use bytes::{Bytes, BytesMut};
use rsocket_rust::error::RSocketError;
use rsocket_rust::extension::{
    CompositeMetadata, CompositeMetadataEntry, MimeType, RoutingMetadata,
};
use rsocket_rust::prelude::*;
use rsocket_rust::utils::Writeable;
use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::LinkedList;
use std::error::Error;
use std::net::SocketAddr;
use std::result::Result;
use std::sync::Arc;
use url::Url;

type FnMetadata = Box<dyn FnMut() -> Result<(MimeType, Vec<u8>), Box<dyn Error>>>;
type FnData = Box<dyn FnMut(&MimeType) -> Result<Vec<u8>, Box<dyn Error>>>;
type PreflightResult = Result<(Payload, MimeType, Arc<Box<dyn RSocket>>), Box<dyn Error>>;

enum TransportKind {
    TCP(String, u16),
    WS(String),
}

pub struct Requester {
    rsocket: Arc<Box<dyn RSocket>>,
}

pub struct RequestSpec {
    rsocket: Arc<Box<dyn RSocket>>,
    data_mime_type: MimeType,
    metadatas: LinkedList<FnMetadata>,
    data: Option<FnData>,
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
        let result = match &self.data_mime_type {
            Some(m) => do_marshal(m, data),
            None => do_marshal(&MimeType::APPLICATION_JSON, data),
        };
        match result {
            Ok(raw) => {
                self.data = Some(raw);
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
        let mime_type = mime_type.into();
        match do_marshal(&mime_type, metadata) {
            Ok(raw) => {
                let entry = CompositeMetadataEntry::new(mime_type, Bytes::from(raw));
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

        let mut metadatas: LinkedList<FnMetadata> = LinkedList::new();
        metadatas.push_back(Box::new(move || {
            Ok((MimeType::MESSAGE_X_RSOCKET_ROUTING_V0, buf.to_vec()))
        }));

        RequestSpec {
            rsocket: self.rsocket.clone(),
            data_mime_type: MimeType::APPLICATION_JSON,
            metadatas,
            data: None,
        }
    }
}

impl RequestSpec {
    pub fn metadata<T, M>(mut self, metadata: T, mime_type: M) -> Self
    where
        T: Sized + Serialize + 'static,
        M: Into<MimeType>,
    {
        let mime_type = mime_type.into();
        let f: FnMetadata = Box::new(move || {
            let raw = do_marshal(&mime_type, &metadata)?;
            Ok((mime_type.clone(), raw))
        });
        self.metadatas.push_back(f);
        self
    }

    pub fn metadata_raw<I, M>(mut self, metadata: I, mime_type: M) -> Self
    where
        I: Into<Vec<u8>>,
        M: Into<MimeType>,
    {
        let mime_type = mime_type.into();
        let metadata = metadata.into();
        self.metadatas
            .push_back(Box::new(move || Ok((mime_type.clone(), metadata.clone()))));
        self
    }

    pub fn data<T>(mut self, data: T) -> Self
    where
        T: Sized + Serialize + 'static,
    {
        self.data = Some(Box::new(move |m| do_marshal(m, &data)));
        self
    }

    pub fn data_raw<I>(mut self, data: I) -> Self
    where
        I: Into<Vec<u8>>,
    {
        let data = data.into();
        self.data = Some(Box::new(move |_m| Ok(data.clone())));
        self
    }

    pub async fn retrieve_mono(self) -> Unpacker {
        match self.preflight() {
            Ok((req, mime_type, rsocket)) => {
                let res = rsocket.request_response(req).await;
                match res {
                    Ok(v) => Unpacker {
                        inner: Ok((mime_type, v)),
                    },
                    Err(e) => Unpacker { inner: Err(e) },
                }
            }
            Err(e) => {
                // TODO: better error
                let msg = format!("{}", e);
                Unpacker {
                    inner: Err(RSocketError::from(msg)),
                }
            }
        }
    }

    #[inline]
    fn preflight(self) -> PreflightResult {
        let mut b = BytesMut::new();
        let mut c = CompositeMetadata::builder();

        for mut b in self.metadatas.into_iter() {
            let (mime_type, raw) = b()?;
            c = c.push(mime_type, raw);
        }
        c.build().write_to(&mut b);

        let mut bu = Payload::builder().set_metadata(b.to_vec());
        if let Some(mut gen) = self.data {
            let raw = gen(&self.data_mime_type)?;
            bu = bu.set_data(raw);
        }
        Ok((bu.build(), self.data_mime_type, self.rsocket))
    }
}

pub struct Unpacker {
    inner: Result<(MimeType, Payload), RSocketError>,
}

impl Unpacker {
    pub fn block<T>(&self) -> Result<Option<T>, Box<dyn Error>>
    where
        T: Sized + DeserializeOwned,
    {
        match &self.inner {
            Ok((mime_type, inner)) => match inner.data() {
                // TODO: support more mime types.
                Some(raw) => match *mime_type {
                    MimeType::APPLICATION_JSON => {
                        let t = unmarshal(misc::json(), &raw.as_ref())?;
                        Ok(Some(t))
                    }
                    MimeType::APPLICATION_CBOR => {
                        let t = unmarshal(misc::cbor(), &raw.as_ref())?;
                        Ok(Some(t))
                    }
                    _ => Err("unsupported mime type!".into()),
                },
                None => Ok(None),
            },
            Err(e) => Err(format!("{}", e).into()),
        }
    }
}

fn do_marshal<T>(mime_type: &MimeType, data: &T) -> Result<Vec<u8>, Box<dyn Error>>
where
    T: Sized + Serialize,
{
    // TODO: support more mime types
    match *mime_type {
        MimeType::APPLICATION_JSON => marshal(misc::json(), data),
        MimeType::APPLICATION_CBOR => marshal(misc::cbor(), data),
        _ => Err("unsupported mime type!".into()),
    }
}
