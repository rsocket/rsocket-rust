use crate::errors::{ErrorKind, RSocketError};
use crate::frame;
use crate::payload::{Payload, SetupPayload};
use crate::result::RSocketResult;
use futures::future;
use futures::{Sink, SinkExt, Stream, StreamExt};
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;
use tokio::sync::mpsc;

// TODO: switch to reactor-rust.
pub type Mono<T> = Pin<Box<dyn Send + Sync + Future<Output = RSocketResult<T>>>>;
pub type Flux<T> = Pin<Box<dyn Send + Sync + Stream<Item = RSocketResult<T>>>>;

pub trait RSocket: Sync + Send {
    fn metadata_push(&self, req: Payload) -> Mono<()>;
    fn fire_and_forget(&self, req: Payload) -> Mono<()>;
    fn request_response(&self, req: Payload) -> Mono<Payload>;
    fn request_stream(&self, req: Payload) -> Flux<Payload>;
    fn request_channel(&self, reqs: Flux<Payload>) -> Flux<Payload>;
}

pub struct EchoRSocket;

impl RSocket for EchoRSocket {
    fn metadata_push(&self, req: Payload) -> Mono<()> {
        info!("{:?}", req);
        Box::pin(future::ok::<(), RSocketError>(()))
    }
    fn fire_and_forget(&self, req: Payload) -> Mono<()> {
        info!("{:?}", req);
        Box::pin(future::ok::<(), RSocketError>(()))
    }
    fn request_response(&self, req: Payload) -> Mono<Payload> {
        info!("{:?}", req);
        Box::pin(future::ok::<Payload, RSocketError>(req))
    }
    fn request_stream(&self, req: Payload) -> Flux<Payload> {
        info!("{:?}", req);
        Box::pin(futures::stream::iter(vec![
            Ok(req.clone()),
            Ok(req.clone()),
            Ok(req),
        ]))
    }
    fn request_channel(&self, mut reqs: Flux<Payload>) -> Flux<Payload> {
        let (sender, receiver) = mpsc::unbounded_channel::<RSocketResult<Payload>>();
        tokio::spawn(async move {
            while let Some(it) = reqs.next().await {
                let pa = it.unwrap();
                info!("{:?}", pa);
                sender.send(Ok(pa)).unwrap();
            }
        });
        Box::pin(receiver)
        // or returns directly
        // reqs
    }
}

pub struct EmptyRSocket;

impl EmptyRSocket {
    fn must_failed(&self) -> RSocketError {
        RSocketError::from(ErrorKind::Internal(frame::ERR_APPLICATION, "NOT_IMPLEMENT"))
    }
}

impl RSocket for EmptyRSocket {
    fn metadata_push(&self, _req: Payload) -> Mono<()> {
        Box::pin(future::err(self.must_failed()))
    }

    fn fire_and_forget(&self, _req: Payload) -> Mono<()> {
        Box::pin(future::err(self.must_failed()))
    }

    fn request_response(&self, _req: Payload) -> Mono<Payload> {
        Box::pin(future::err(self.must_failed()))
    }

    fn request_stream(&self, _req: Payload) -> Flux<Payload> {
        Box::pin(futures::stream::iter(vec![Err(self.must_failed())]))
    }
    fn request_channel(&self, _reqs: Flux<Payload>) -> Flux<Payload> {
        Box::pin(futures::stream::iter(vec![Err(self.must_failed())]))
    }
}

pub type AcceptorGenerator = Arc<fn(SetupPayload, Box<dyn RSocket>) -> Box<dyn RSocket>>;

pub enum Acceptor {
    Direct(Box<dyn RSocket>),
    Generate(AcceptorGenerator),
    Empty(),
}
