# RSocket Transport For TCP

## Example

Add dependencies in your `Cargo.toml`.

```toml
[dependencies]
tokio = "0.3.6"
rsocket_rust = "0.7.0"
rsocket_rust_transport_tcp = "0.7.0"
```

### Server

```rust
use log::info;
use rsocket_rust::prelude::{RSocketFactory, ServerResponder};
use rsocket_rust::Result;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust_transport_tcp::TcpServerTransport;

#[tokio::main]
async fn main() -> Result<()> {
    let transport: TcpServerTransport = TcpServerTransport::from("127.0.0.1:7878");

    let responder: ServerResponder = Box::new(|setup, _socket| {
        info!("accept setup: {:?}", setup);
        Ok(Box::new(EchoRSocket))
        // Or you can reject setup
        // Err(From::from("SETUP_NOT_ALLOW"))
    });

    let on_start: Box<dyn FnMut() + Send + Sync> =
        Box::new(|| info!("+++++++ echo server started! +++++++"));

    RSocketFactory::receive()
        .transport(transport)
        .acceptor(responder)
        .on_start(on_start)
        .serve()
        .await?;

    Ok(())
}

```

### Client

```rust
use log::info;
use rsocket_rust::prelude::{ClientResponder, Payload, RSocket, RSocketFactory};
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::TcpClientTransport;

#[tokio::main]
async fn main() -> Result<()> {
    let client = RSocketFactory::connect()
        .transport(TcpClientTransport::from("127.0.0.1:7878"))
        .setup(Payload::from("READY!"))
        .mime_type("text/plain", "text/plain")
        .start()
        .await?;

    let request_payload: Payload = Payload::builder()
        .set_data_utf8("Hello World!")
        .set_metadata_utf8("Rust")
        .build();

    let res = client.request_response(request_payload).await?;

    info!("got: {:?}", res);

    client.close();

    Ok(())
}

```
