use super::{Client, ClientBuilder, ServerBuilder};

pub struct RSocketFactory;

impl RSocketFactory {
    pub fn connect<'a>() -> ClientBuilder<'a> {
        Client::builder()
    }
    pub fn receive<'a>() -> ServerBuilder<'a> {
        ServerBuilder::new()
    }
}
