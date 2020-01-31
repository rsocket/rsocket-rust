use super::client::WebsocketClientTransport;
use rsocket_rust::transport::ServerTransport;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use url::Url;

pub struct WebsocketServerTransport {
    addr: Url,
}

impl ServerTransport for WebsocketServerTransport {
    type Item = WebsocketClientTransport;

    fn start(
        self,
        starter: Option<fn()>,
        acceptor: impl Fn(WebsocketClientTransport) + Send + Sync + 'static,
    ) -> Pin<Box<dyn Sync + Send + Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>> {
        // TODO: see https://github.com/snapview/tokio-tungstenite/blob/master/examples/server.rs
        unimplemented!()
    }
}
