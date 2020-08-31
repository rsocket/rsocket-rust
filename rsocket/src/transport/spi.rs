use crate::frame::Frame;
use crate::payload::SetupPayload;
use crate::spi::{ClientResponder, RSocket, ServerResponder};
use crate::{Error, Result};
use async_trait::async_trait;
use futures::channel::{mpsc, oneshot};
use futures::sink::Sink;
use futures::stream::Stream;
use std::future::Future;
use std::marker::Unpin;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::Notify;

#[derive(Clone)]
pub(crate) enum Acceptor {
    Simple(Arc<ClientResponder>),
    Generate(Arc<ServerResponder>),
}

#[async_trait]
pub trait Reader {
    async fn read(&mut self) -> Option<Result<Frame>>;
}

#[async_trait]
pub trait Writer {
    async fn write(&mut self, frame: Frame) -> Result<()>;
}

pub trait Connection {
    fn split(
        self,
    ) -> (
        Box<dyn Writer + Send + Unpin>,
        Box<dyn Reader + Send + Unpin>,
    );
}

#[async_trait]
pub trait Transport {
    type Conn: Connection + Send;

    async fn connect(self) -> Result<Self::Conn>;
}

#[async_trait]
pub trait ServerTransportOld {
    type Item;

    async fn start(
        self,
        starter: Option<Box<dyn FnMut() + Send + Sync>>,
        acceptor: Box<dyn Fn(Self::Item) -> Result<()> + Send + Sync>,
    ) -> Result<()>
    where
        Self::Item: Transport + Sized;
}

#[async_trait]
pub trait ServerTransport {
    type Item: Transport;

    async fn start(&mut self) -> Result<()>;

    async fn next(&mut self) -> Option<Result<Self::Item>>;
}
