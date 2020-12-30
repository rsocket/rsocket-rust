use crate::error::{self, RSocketError};
use crate::payload::{Payload, SetupPayload};
use crate::{runtime, Error, Result};
use async_trait::async_trait;
use futures::Stream;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;

pub type ClientResponder = Box<dyn Send + Sync + Fn() -> Box<dyn RSocket>>;
pub type ServerResponder =
    Box<dyn Send + Sync + Fn(SetupPayload, Box<dyn RSocket>) -> Result<Box<dyn RSocket>>>;

pub type Flux<T> = Pin<Box<dyn Send + Stream<Item = T>>>;

#[async_trait]
pub trait RSocket: Sync + Send {
    async fn metadata_push(&self, req: Payload) -> Result<()>;
    async fn fire_and_forget(&self, req: Payload) -> Result<()>;
    async fn request_response(&self, req: Payload) -> Result<Option<Payload>>;
    fn request_stream(&self, req: Payload) -> Flux<Result<Payload>>;
    fn request_channel(&self, reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>>;
}
