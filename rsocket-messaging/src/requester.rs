use super::misc::{self, marshal, unmarshal};
use bytes::{Bytes, BytesMut};
use rsocket_rust::extension::{CompositeMetadata, MimeType, RoutingMetadata};
use rsocket_rust::prelude::*;
use rsocket_rust::utils::Writeable;
use rsocket_rust::{error::RSocketError, Result};
use rsocket_rust_transport_tcp::TcpClientTransport;
use rsocket_rust_transport_websocket::WebsocketClientTransport;
use serde::{de::DeserializeOwned, Serialize};
use std::collections::LinkedList;
use std::net::SocketAddr;
use std::sync::Arc;
use url::Url;

type FnMetadata = Box<dyn FnMut() -> Result<(MimeType, Vec<u8>)>>;
type FnData = Box<dyn FnMut(&MimeType) -> Result<Vec<u8>>>;
type PreflightResult = Result<(Payload, MimeType, Arc<Box<dyn RSocket>>)>;
type UnpackerResult = Result<(MimeType, Payload)>;
type UnpackersResult = Result<(MimeType, Flux<Result<Payload>>)>;

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
    metadata: LinkedList<FnMetadata>,
    data: Option<FnData>,
    tp: Option<TransportKind>,
}

pub struct Unpackers {
    inner: UnpackersResult,
}

pub struct Unpacker {
    inner: UnpackerResult,
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

    pub fn setup_data<D>(mut self, data: D) -> Self
    where
        D: Sized + Serialize + 'static,
    {
        self.data = Some(Box::new(move |mime_type: &MimeType| {
            do_marshal(mime_type, &data)
        }));
        self
    }

    pub fn setup_metadata<M, T>(mut self, metadata: M, mime_type: T) -> Self
    where
        M: Sized + Serialize + 'static,
        T: Into<MimeType>,
    {
        let mime_type = mime_type.into();
        self.metadata.push_back(Box::new(move || {
            let raw = do_marshal(&mime_type, &metadata)?;
            Ok((mime_type.clone(), raw))
        }));
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

    pub async fn build(self) -> Result<Requester> {
        let data_mime_type = self.data_mime_type.unwrap_or(MimeType::APPLICATION_JSON);

        let mut added = 0usize;
        let mut composite_builder = CompositeMetadata::builder();

        if let Some(s) = self.route {
            let routing = RoutingMetadata::builder().push(s).build();
            composite_builder =
                composite_builder.push(MimeType::MESSAGE_X_RSOCKET_ROUTING_V0, routing.bytes());
            added += 1;
        }

        for mut gen in self.metadata.into_iter() {
            let (mime_type, raw) = gen()?;
            composite_builder = composite_builder.push(mime_type, raw);
            added += 1;
        }

        let mut payload_builder = Payload::builder();

        if added > 0 {
            payload_builder = payload_builder.set_metadata(composite_builder.build());
        }

        if let Some(mut gen) = self.data {
            payload_builder = payload_builder.set_data(gen(&data_mime_type)?);
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
            None => Err(RSocketError::WithDescription("Missing transport!".into()).into()),
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

    pub async fn retrieve(self) -> Result<()> {
        let (req, _mime_type, rsocket) = self.preflight()?;
        rsocket.fire_and_forget(req).await;
        Ok(())
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
            Err(e) => Unpacker { inner: Err(e) },
        }
    }

    pub fn retrieve_flux(self) -> Unpackers {
        match self.preflight() {
            Ok((req, mime_type, rsocket)) => {
                let results = rsocket.request_stream(req);
                Unpackers {
                    inner: Ok((mime_type, results)),
                }
            }
            Err(e) => Unpackers { inner: Err(e) },
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

impl Unpackers {
    pub async fn block<T>(self) -> Result<Vec<T>>
    where
        T: Sized + DeserializeOwned,
    {
        let (mime_type, mut results) = self.inner?;
        let mut res = Vec::new();
        while let Some(next) = results.next().await {
            let v = next?;
            if let Some(data) = v.data() {
                let t = do_unmarshal::<T>(&mime_type, data)?;
                if let Some(t) = t {
                    res.push(t);
                }
            }
        }
        Ok(res)
    }

    pub async fn foreach<T>(self, callback: impl Fn(T)) -> Result<()>
    where
        T: Sized + DeserializeOwned,
    {
        let (mime_type, mut results) = self.inner?;
        while let Some(next) = results.next().await {
            let v = next?;
            if let Some(data) = v.data() {
                let t = do_unmarshal::<T>(&mime_type, data)?;
                if let Some(t) = t {
                    callback(t);
                }
            }
        }
        Ok(())
    }
}

impl Unpacker {
    pub fn block<T>(self) -> Result<Option<T>>
    where
        T: Sized + DeserializeOwned,
    {
        let (mime_type, inner) = self.inner?;
        match inner.data() {
            Some(raw) => do_unmarshal(&mime_type, raw),
            None => Ok(None),
        }
    }
}

fn do_unmarshal<T>(mime_type: &MimeType, raw: &Bytes) -> Result<Option<T>>
where
    T: Sized + DeserializeOwned,
{
    // TODO: support more mime types
    match *mime_type {
        MimeType::APPLICATION_JSON => Ok(Some(unmarshal(misc::json(), &raw.as_ref())?)),
        MimeType::APPLICATION_CBOR => Ok(Some(unmarshal(misc::cbor(), &raw.as_ref())?)),
        _ => Err(RSocketError::WithDescription("unsupported mime type!".into()).into()),
    }
}

fn do_marshal<T>(mime_type: &MimeType, data: &T) -> Result<Vec<u8>>
where
    T: Sized + Serialize,
{
    // TODO: support more mime types
    match *mime_type {
        MimeType::APPLICATION_JSON => marshal(misc::json(), data),
        MimeType::APPLICATION_CBOR => marshal(misc::cbor(), data),
        _ => Err(RSocketError::WithDescription("unsupported mime type!".into()).into()),
    }
}
