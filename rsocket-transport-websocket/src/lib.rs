#![allow(clippy::type_complexity)]

#[macro_use]
extern crate log;

mod client;
mod connection;
mod server;

pub use client::{WebsocketClientTransport, WebsocketRequest};
pub use server::WebsocketServerTransport;

#[cfg(test)]
mod test_websocket {
    use super::*;
    use rsocket_rust::prelude::*;

    #[ignore]
    #[tokio::test]
    async fn test_client() {
        let req: WebsocketRequest = WebsocketRequest::builder()
            .uri("ws://127.0.0.1:8080/hello")
            .header("x-foo-bar", "42")
            .method("GET")
            .body(())
            .unwrap();
        let tp = WebsocketClientTransport::from(req);
        let c = RSocketFactory::connect()
            .transport(tp)
            .start()
            .await
            .expect("connect failed");
        let res = c
            .request_response(Payload::builder().set_data_utf8("foo").build())
            .await
            .expect("request failed")
            .unwrap();

        println!("response: {:?}", res);
    }
}
