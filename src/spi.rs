use crate::errors::{self, ErrorKind, RSocketError};
use crate::frame;
use crate::payload::{Payload, SetupPayload};
use crate::result::RSocketResult;

use futures::future;
use futures::{Sink, SinkExt, Stream, StreamExt};
use std::future::Future;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

// TODO: switch to reactor-rust.
pub type Mono<T> = Pin<Box<dyn Send + Sync + Future<Output = T>>>;
pub type Flux<T> = Pin<Box<dyn Send + Sync + Stream<Item = T>>>;

pub trait RSocket: Sync + Send {
    fn metadata_push(&self, req: Payload) -> Mono<()>;
    fn fire_and_forget(&self, req: Payload) -> Mono<()>;
    fn request_response(&self, req: Payload) -> Mono<Result<Payload, RSocketError>>;
    fn request_stream(&self, req: Payload) -> Flux<Payload>;
    fn request_channel(&self, reqs: Flux<Payload>) -> Flux<Payload>;
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
    fn request_response(&self, req: Payload) -> Mono<Result<Payload, RSocketError>> {
        info!("{:?}", req);
        Box::pin(async move { Ok(req) })
    }
    fn request_stream(&self, req: Payload) -> Flux<Payload> {
        info!("{:?}", req);
        // repeat 3 times.
        Box::pin(futures::stream::iter(vec![req.clone(), req.clone(), req]))
    }
    fn request_channel(&self, mut reqs: Flux<Payload>) -> Flux<Payload> {
        let (sender, receiver) = mpsc::unbounded_channel::<Payload>();
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

impl EmptyRSocket {
    fn must_failed(&self) -> RSocketError {
        let kind = ErrorKind::Internal(errors::ERR_APPLICATION, String::from("NOT_IMPLEMENT"));
        RSocketError::from(kind)
    }
}

impl RSocket for EmptyRSocket {
    fn metadata_push(&self, _req: Payload) -> Mono<()> {
        Box::pin(async {})
    }

    fn fire_and_forget(&self, _req: Payload) -> Mono<()> {
        Box::pin(async {})
    }

    fn request_response(&self, _req: Payload) -> Mono<Result<Payload, RSocketError>> {
        Box::pin(future::err(self.must_failed()))
    }

    fn request_stream(&self, _req: Payload) -> Flux<Payload> {
        Box::pin(futures::stream::empty())
    }
    fn request_channel(&self, _reqs: Flux<Payload>) -> Flux<Payload> {
        Box::pin(futures::stream::empty())
    }
}
