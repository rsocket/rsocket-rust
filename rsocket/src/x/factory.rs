use super::{Client, ClientBuilder, ServerBuilder};
use crate::transport::{ClientTransport, ServerTransport};

pub struct RSocketFactory;

impl RSocketFactory {
    pub fn connect<T>() -> ClientBuilder<T>
    where
        T: Send + Sync + ClientTransport,
    {
        ClientBuilder::new()
    }

    pub fn receive<T, C>() -> ServerBuilder<T, C>
    where
        T: Send + Sync + ServerTransport<Item = C> + 'static,
        C: Send + Sync + ClientTransport + 'static,
    {
        ServerBuilder::new()
    }
}
