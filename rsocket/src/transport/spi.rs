use std::future::Future;
use std::io::Error as IOError;
use std::marker::Unpin;
use std::pin::Pin;
use std::result::Result as StdResult;
use std::sync::Arc;

use async_trait::async_trait;
use futures::channel::{mpsc, oneshot};
use futures::{Sink, Stream};
use tokio::sync::Notify;

use crate::payload::SetupPayload;
use crate::spi::{ClientResponder, RSocket, ServerResponder};
use crate::{error::RSocketError, frame::Frame};
use crate::{Error, Result};

#[derive(Clone)]
pub(crate) enum Acceptor {
    Simple(Arc<ClientResponder>),
    Generate(Arc<ServerResponder>),
}

pub type FrameSink = dyn Sink<Frame, Error = RSocketError> + Send + Unpin;
pub type FrameStream = dyn Stream<Item = StdResult<Frame, RSocketError>> + Send + Unpin;

pub trait Connection {
    fn split(self) -> (Box<FrameSink>, Box<FrameStream>);
}

#[async_trait]
pub trait Transport {
    type Conn: Connection + Send;

    async fn connect(self) -> Result<Self::Conn>;
}

#[async_trait]
pub trait ServerTransport {
    type Item: Transport;

    async fn start(&mut self) -> Result<()>;

    async fn next(&mut self) -> Option<Result<Self::Item>>;
}
