use super::{Client, ClientBuilder, ServerBuilder};
use crate::transport::{Connection, ServerTransport, Transport};

#[derive(Debug)]
pub struct RSocketFactory;

impl RSocketFactory {
    pub fn connect<T, C>() -> ClientBuilder<T, C>
    where
        T: Send + Sync + Transport<Conn = C>,
        C: Send + Sync + Connection,
    {
        ClientBuilder::new()
    }

    pub fn receive<S, T>() -> ServerBuilder<S, T>
    where
        S: Send + Sync + ServerTransport<Item = T>,
        T: Send + Sync + Transport,
    {
        ServerBuilder::new()
    }
}
