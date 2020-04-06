# RSocket Core

## Example

> Here are some example codes which show how RSocket works in Rust.

### Dependencies

Add dependencies in your `Cargo.toml`.

```toml
[dependencies]
tokio = "0.2"
rsocket_rust = "*"

# choose transport:
# rsocket_rust_transport_tcp = "*"
# rsocket_rust_transport_websocket = "*"
```

### Server

```rust
#[macro_use]
extern crate log;

use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpServerTransport;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::builder().format_timestamp_millis().init();
    let addr = env::args().nth(1).unwrap_or("127.0.0.1:7878".to_string());

    RSocketFactory::receive()
        .transport(TcpServerTransport::from(addr))
        .acceptor(Box::new(|setup, _socket| {
            info!("accept setup: {:?}", setup);
            Ok(Box::new(EchoRSocket))
            // Or you can reject setup
            // Err(From::from("SETUP_NOT_ALLOW"))
        }))
        .on_start(|| info!("+++++++ echo server started! +++++++"))
        .serve()
        .await
}
```

### Client

```rust
use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpClientTransport;

#[tokio::main]
#[test]
async fn test() {
    let cli = RSocketFactory::connect()
        .acceptor(Box::new(|| Box::new(EchoRSocket)))
        .transport(TcpClientTransport::from("127.0.0.1:7878"))
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
