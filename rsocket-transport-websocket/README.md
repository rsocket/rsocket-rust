# RSocket Transport For Websocket

## Example

Add dependencies in your `Cargo.toml`.

```toml
[dependencies]
tokio = "0.2.11"
rsocket_rust = "0.5.0"

# choose transport:
# rsocket_rust_transport_tcp = "0.5.0"
# rsocket_rust_transport_websocket = "0.5.0"
```

### Server

```rust
use rsocket_rust::prelude::*;
use rsocket_rust_transport_websocket::WebsocketServerTransport;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    RSocketFactory::receive()
        .transport(WebsocketServerTransport::from("127.0.0.1:8080"))
        .acceptor(|setup, _socket| {
            println!("accept setup: {:?}", setup)
            Ok(Box::new(EchoRSocket))
            // Or you can reject setup
            // Err(From::from("SETUP_NOT_ALLOW"))
        })
        .on_start(|| println!("+++++++ echo server started! +++++++"))
        .serve()
        .await
}

```

### Client

```rust
use rsocket_rust::prelude::*;
use rsocket_rust_transport_websocket::WebsocketClientTransport;

#[tokio::main]
#[test]
async fn test() {
    let cli = RSocketFactory::connect()
        .acceptor(|| Box::new(EchoRSocket))
        .transport(WebsocketClientTransport::from("127.0.0.1:8080"))
        .setup(Payload::from("READY!"))
        .mime_type("text/plain", "text/plain")
        .start()
        .await
        .unwrap();
    let req = Payload::builder()
        .set_data_utf8("Hello World!")
        .set_metadata_utf8("Rust")
        .build();
    let res = cli.request_response(req).await.unwrap();
    println!("got: {:?}", res);
    cli.close();
}

```
