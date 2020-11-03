use crate::error::{self, RSocketError};
use crate::payload::{Payload, SetupPayload};
use crate::{runtime, Error, Result};
use futures::future;
use futures::{pin_mut, FutureExt, Sink, SinkExt, Stream, StreamExt};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;

pub type ClientResponder = Box<dyn Send + Sync + Fn() -> Box<dyn RSocket>>;
pub type ServerResponder =
    Box<dyn Send + Sync + Fn(SetupPayload, Box<dyn RSocket>) -> Result<Box<dyn RSocket>>>;

pub type Mono<T> = Pin<Box<dyn Send + Future<Output = T>>>;
pub type Flux<T> = Pin<Box<dyn Send + Stream<Item = T>>>;

pub trait RSocket: Sync + Send {
    fn metadata_push(&self, req: Payload) -> Mono<()>;
    fn fire_and_forget(&self, req: Payload) -> Mono<()>;
    fn request_response(&self, req: Payload) -> Mono<Result<Payload>>;
    fn request_stream(&self, req: Payload) -> Flux<Result<Payload>>;
    fn request_channel(&self, reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>>;
}

pub struct EchoRSocket;

impl RSocket for EchoRSocket {
    fn metadata_push(&self, req: Payload) -> Mono<()> {
        info!("{:?}", req);
        Box::pin(async {})
    }

    fn fire_and_forget(&self, req: Payload) -> Mono<()> {
        info!("{:?}", req);
        Box::pin(async {})
    }

    fn request_response(&self, req: Payload) -> Mono<Result<Payload>> {
        info!("{:?}", req);
        Box::pin(async move { Ok(req) })
    }

    fn request_stream(&self, req: Payload) -> Flux<Result<Payload>> {
        info!("{:?}", req);
        // repeat 3 times.
        Box::pin(futures::stream::iter(vec![
            Ok(req.clone()),
            Ok(req.clone()),
            Ok(req),
        ]))
    }

    fn request_channel(&self, mut reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        let (sender, receiver) = mpsc::unbounded_channel();
        tokio::spawn(async move {
            while let Some(it) = reqs.next().await {
                info!("{:?}", it);
                sender.send(it).unwrap();
            }
        });
        Box::pin(receiver)
        // or returns directly
        // reqs
    }
}

pub(crate) struct EmptyRSocket;

impl RSocket for EmptyRSocket {
    fn metadata_push(&self, _req: Payload) -> Mono<()> {
        Box::pin(async {})
    }

    fn fire_and_forget(&self, _req: Payload) -> Mono<()> {
        Box::pin(async {})
    }

    fn request_response(&self, _req: Payload) -> Mono<Result<Payload>> {
        Box::pin(future::err(
            RSocketError::ApplicationException("NOT_IMPLEMENT".into()).into(),
        ))
    }

    fn request_stream(&self, _req: Payload) -> Flux<Result<Payload>> {
        Box::pin(futures::stream::empty())
    }

    fn request_channel(&self, _reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        Box::pin(futures::stream::empty())
    }
}
