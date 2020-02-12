use crate::error::{self, ErrorKind, RSocketError};
use crate::frame;
use crate::payload::Payload;
use crate::utils::RSocketResult;

use futures::future;
use futures::{Sink, SinkExt, Stream, StreamExt};
use std::future::Future;
use std::pin::Pin;
use std::result::Result;
use std::sync::Arc;
use tokio::sync::mpsc;

pub type Mono<T> = Pin<Box<dyn Send + Sync + Future<Output = T>>>;
pub type Flux<T> = Pin<Box<dyn Send + Sync + Stream<Item = T>>>;

pub trait RSocket: Sync + Send {
    fn metadata_push(&self, req: Payload) -> Mono<()>;
    fn fire_and_forget(&self, req: Payload) -> Mono<()>;
    fn request_response(&self, req: Payload) -> Mono<Result<Payload, RSocketError>>;
    fn request_stream(&self, req: Payload) -> Flux<Result<Payload, RSocketError>>;
    fn request_channel(
        &self,
        reqs: Flux<Result<Payload, RSocketError>>,
    ) -> Flux<Result<Payload, RSocketError>>;
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

    fn request_stream(&self, req: Payload) -> Flux<Result<Payload, RSocketError>> {
        info!("{:?}", req);
        // repeat 3 times.
        Box::pin(futures::stream::iter(vec![
            Ok(req.clone()),
            Ok(req.clone()),
            Ok(req),
        ]))
    }

    fn request_channel(
        &self,
        mut reqs: Flux<Result<Payload, RSocketError>>,
    ) -> Flux<Result<Payload, RSocketError>> {
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

impl EmptyRSocket {
    fn must_failed(&self) -> RSocketError {
        let kind = ErrorKind::Internal(error::ERR_APPLICATION, String::from("NOT_IMPLEMENT"));
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

    fn request_stream(&self, _req: Payload) -> Flux<Result<Payload, RSocketError>> {
        Box::pin(futures::stream::empty())
    }

    fn request_channel(
        &self,
        _reqs: Flux<Result<Payload, RSocketError>>,
    ) -> Flux<Result<Payload, RSocketError>> {
        Box::pin(futures::stream::empty())
    }
}
