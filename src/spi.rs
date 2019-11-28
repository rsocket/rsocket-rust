use crate::{
    errors::{ErrorKind, RSocketError},
    frame,
    payload::{Payload, SetupPayload},
    result::RSocketResult,
};
use futures::future;
use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

pub type Single<T> = Pin<Box<dyn Send + Sync + Future<Output = RSocketResult<T>>>>;

pub trait RSocket: Sync + Send {
    fn metadata_push(&self, req: Payload) -> Single<()>;
    fn fire_and_forget(&self, req: Payload) -> Single<()>;
    fn request_response(&self, req: Payload) -> Single<Payload>;
}

pub struct EchoRSocket;

impl RSocket for EchoRSocket {
    fn metadata_push(&self, req: Payload) -> Single<()> {
        Box::pin(future::ok::<(), RSocketError>(()))
    }
    fn fire_and_forget(&self, req: Payload) -> Single<()> {
        info!("echo fire_and_forget: {:?}", req);
        Box::pin(future::ok::<(), RSocketError>(()))
    }

    fn request_response(&self, req: Payload) -> Single<Payload> {
        info!("echo request_response: {:?}", req);
        Box::pin(future::ok::<Payload, RSocketError>(req))
    }
}

pub struct EmptyRSocket;

impl EmptyRSocket {
    fn must_failed(&self) -> RSocketError {
        RSocketError::from(ErrorKind::Internal(frame::ERR_APPLICATION, "NOT_IMPLEMENT"))
    }
}

impl RSocket for EmptyRSocket {
    fn metadata_push(&self, _req: Payload) -> Single<()> {
        Box::pin(future::err(self.must_failed()))
    }

    fn fire_and_forget(&self, _req: Payload) -> Single<()> {
        Box::pin(future::err(self.must_failed()))
    }

    fn request_response(&self, _req: Payload) -> Single<Payload> {
        Box::pin(future::err(self.must_failed()))
    }
}

pub type AcceptorGenerator = Arc<fn(SetupPayload, Box<dyn RSocket>) -> Box<dyn RSocket>>;

pub enum Acceptor {
    Direct(Box<dyn RSocket>),
    Generate(AcceptorGenerator),
    Empty(),
}
