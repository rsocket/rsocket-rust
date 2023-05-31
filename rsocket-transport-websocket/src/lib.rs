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
    use rsocket_rust::prelude::*;
    use tokio_tungstenite::tungstenite::client::IntoClientRequest;

    use super::*;

    #[ignore]
    #[tokio::test]
    async fn test_client() {
        let mut req = "ws://127.0.0.1:8080/hello".into_client_request().unwrap();

        // Optional: custom headers
        let headers = req.headers_mut();
        headers.insert("x-foo-bar", "42".parse().unwrap());

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
