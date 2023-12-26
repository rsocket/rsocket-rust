use std::future::Future;
use std::pin::Pin;

use async_trait::async_trait;
use futures::Stream;

use crate::payload::{Payload, SetupPayload};
use crate::Result;

pub type ClientResponder = Box<dyn Send + Sync + FnOnce() -> Box<dyn RSocket>>;
pub type ServerResponder =
    Box<dyn Send + Sync + Fn(SetupPayload, Box<dyn RSocket>) -> Result<Box<dyn RSocket>>>;

pub type Flux<T> = Pin<Box<dyn Send + Stream<Item = T>>>;

/// A contract providing different interaction models for RSocket protocol.
///
/// RSocket trait is based on `async_trait` crate.
///
/// # Example
/// ```
/// use rsocket_rust::prelude::*;
/// use rsocket_rust::{async_trait, stream, Result};
///
/// struct ExampleRSocket;
///
/// #[async_trait]
/// impl RSocket for ExampleRSocket {
///     async fn metadata_push(&self, req: Payload) -> Result<()> {
///         Ok(())
///     }
///
///     async fn fire_and_forget(&self, req: Payload) -> Result<()> {
///         Ok(())
///     }
///
///     async fn request_response(&self, req: Payload) -> Result<Option<Payload>> {
///         Ok(Some(Payload::builder().set_data_utf8("bingo").build()))
///     }
///
///     fn request_stream(&self, req: Payload) -> Flux<Result<Payload>> {
///         Box::pin(stream! {
///             for _ in 0..3 {
///                 yield Ok(Payload::builder().set_data_utf8("next payload").build());
///             }
///         })
///     }
///
///     fn request_channel(&self, reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
///         reqs
///     }
/// }
/// ```
#[async_trait]
pub trait RSocket: Sync + Send {
    /// Metadata-Push interaction model of RSocket.
    async fn metadata_push(&self, req: Payload) -> Result<()>;
    /// Fire and Forget interaction model of RSocket.
    async fn fire_and_forget(&self, req: Payload) -> Result<()>;
    /// Request-Response interaction model of RSocket.
    async fn request_response(&self, req: Payload) -> Result<Option<Payload>>;
    /// Request-Stream interaction model of RSocket.
    fn request_stream(&self, req: Payload) -> Flux<Result<Payload>>;
    /// Request-Channel interaction model of RSocket.
    fn request_channel(&self, reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>>;
}
