# rsocket-rust

![GitHub Workflow Status](https://github.com/rsocket/rsocket-rust/workflows/Rust/badge.svg)
[![Build Status](https://travis-ci.com/rsocket/rsocket-rust.svg?branch=master)](https://travis-ci.com/rsocket/rsocket-rust)
[![Crates.io](https://img.shields.io/crates/v/rsocket_rust)](https://crates.io/crates/rsocket_rust)
[![Crates.io](https://img.shields.io/crates/d/rsocket_rust)](https://crates.io/crates/rsocket_rust)
[![License](https://img.shields.io/github/license/rsocket/rsocket-rust.svg)](https://github.com/rsocket/rsocket-rust/blob/master/LICENSE)
[![GitHub Release](https://img.shields.io/github/release-pre/rsocket/rsocket-rust.svg)](https://github.com/rsocket/rsocket-rust/releases)

> rsocket-rust is an implementation of the RSocket protocol in Rust(1.39+).
It's an **alpha** version and still under active development.
**Do not use it in a production environment!**

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
}

```

## TODO

- Operations
  - [x] METADATA_PUSH
  - [x] REQUEST_FNF
  - [x] REQUEST_RESPONSE
  - [x] REQUEST_STREAM
  - [x] REQUEST_CHANNEL
- More Operations
  - [x] Error
  - [ ] Cancel
  - [x] Fragmentation
  - [ ] Resume
  - [x] Keepalive
- QoS
  - [ ] RequestN
  - [ ] Lease
- Transport
  - [x] TCP
  - [x] Websocket
  - [x] WASM
- Reactor
  - [ ] ...
- High Level APIs
  - [x] Client
  - [x] Server
