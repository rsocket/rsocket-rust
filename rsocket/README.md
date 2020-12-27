# RSocket Core

## Example

> Here are some example codes which show how RSocket works in Rust.

### Dependencies

Add dependencies in your `Cargo.toml`.

```toml
[dependencies]
tokio = "0.3.6"
rsocket_rust = "0.7.0"

# add transport dependencies:
# rsocket_rust_transport_tcp = "0.7.0"
# rsocket_rust_transport_websocket = "0.7.0"
```

### Server

```rust
use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::TcpServerTransport;

#[tokio::main]
async fn main() -> Result<()> {
    RSocketFactory::receive()
        .transport(TcpServerTransport::from("127.0.0.1:7878"))
        .acceptor(Box::new(|setup, _socket| {
            println!("accept setup: {:?}", setup);
            // Use EchoRSocket as example RSocket, you can implement RSocket trait.
            Ok(Box::new(EchoRSocket))
            // Or you can reject setup
            // Err(From::from("SETUP_NOT_ALLOW"))
        }))
        .on_start(|| println!("+++++++ echo server started! +++++++"))
        .serve()
        .await
}
```

### Client

```rust
use rsocket_rust::prelude::*;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::TcpClientTransport;

#[tokio::main]
async fn main() -> Result<()> {
    let cli = RSocketFactory::connect()
        .transport(TcpClientTransport::from("127.0.0.1:7878"))
        .setup(Payload::from("READY!"))
        .mime_type("text/plain", "text/plain")
        .start()
        .await?;
    let req = Payload::builder()
        .set_data_utf8("Hello World!")
        .set_metadata_utf8("Rust")
        .build();
    let res = cli.request_response(req).await?;
    println!("got: {:?}", res);
    cli.close();
    Ok(())
}

```
