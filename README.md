# rsocket-rust

![GitHub Workflow Status](https://github.com/rsocket/rsocket-rust/workflows/Rust/badge.svg)
[![Build Status](https://travis-ci.com/rsocket/rsocket-rust.svg?branch=master)](https://travis-ci.com/rsocket/rsocket-rust)
[![Crates.io](https://img.shields.io/crates/v/rsocket_rust)](https://crates.io/crates/rsocket_rust)
[![Crates.io](https://img.shields.io/crates/d/rsocket_rust)](https://crates.io/crates/rsocket_rust)
[![License](https://img.shields.io/github/license/rsocket/rsocket-rust.svg)](https://github.com/rsocket/rsocket-rust/blob/master/LICENSE)
[![GitHub Release](https://img.shields.io/github/release-pre/rsocket/rsocket-rust.svg)](https://github.com/rsocket/rsocket-rust/releases)

> rsocket-rust is an implementation of the RSocket protocol in Rust(1.39+). It's
> an **alpha** version and still under active development. **Do not use it in a
> production environment!**

## Example

> Here are some example codes which show how RSocket works in Rust.

### Dependencies

Add dependencies in your `Cargo.toml`.

```toml
[dependencies]
tokio = "1.0.3"
rsocket_rust = "0.7"

# add transport dependencies:
# rsocket_rust_transport_tcp = "0.7"
# rsocket_rust_transport_websocket = "0.7"
```

### Server

```rust
extern crate log;

use futures::executor::block_on;
use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::*;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_timestamp_millis().init();

    RSocketFactory::receive()
        .transport(TcpServerTransport::from("127.0.0.1:7979"))
        .acceptor(Box::new(|setup, _sending_socket| {
            info!("incoming socket: setup={:?}", setup);
            Ok(Box::new(block_on(async move {
                RSocketFactory::connect()
                    .transport(TcpClientTransport::from("127.0.0.1:7878"))
                    .acceptor(Box::new(|| Box::new(EchoRSocket)))
                    .setup(Payload::from("I'm Rust!"))
                    .start()
                    .await
                    .unwrap()
            })))
        }))
        .serve()
        .await
}
```

### Client

```rust
extern crate log;

use rsocket_rust::prelude::*;
use rsocket_rust::utils::EchoRSocket;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::TcpClientTransport;

#[tokio::main]
async fn main() -> Result<()> {
     env_logger::builder().format_timestamp_millis().init();
    let client = RSocketFactory::connect()
        .transport(TcpClientTransport::from("127.0.0.1:7878"))
        .acceptor(Box::new(|| {
            // Return a responder.
            Box::new(EchoRSocket)
        }))
        .start()
        .await
        .expect("Connect failed!");

    let req = Payload::builder().set_data_utf8("Ping!").build();

    match client.request_response(req).await {
        Ok(res) => info!("{:?}", res),
        Err(e) => error!("{}", e),
    }

    Ok(())
}
```

### Implement RSocket trait

Example for access Redis([crates](https://crates.io/crates/redis)):

> NOTICE: add dependency in Cargo.toml => redis = { version = "0.19.0", features
> = [ "aio" ] }

```rust
use std::str::FromStr;

use redis::Client as RedisClient;
use rsocket_rust::async_trait;
use rsocket_rust::prelude::*;
use rsocket_rust::Result;

#[derive(Clone)]
pub struct RedisDao {
    inner: RedisClient,
}

// Create RedisDao from str.
// Example: RedisDao::from_str("redis://127.0.0.1").expect("Connect redis failed!");
impl FromStr for RedisDao {
    type Err = redis::RedisError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let client = redis::Client::open(s)?;
        Ok(RedisDao { inner: client })
    }
}

#[async_trait]
impl RSocket for RedisDao {
    async fn request_response(&self, req: Payload) -> Result<Option<Payload>> {
        let client = self.inner.clone();
        let mut conn = client.get_async_connection().await?;
        let value: redis::RedisResult<Option<String>> = redis::cmd("GET")
            .arg(&[req.data_utf8()])
            .query_async(&mut conn)
            .await;
        match value {
            Ok(Some(value)) => Ok(Some(Payload::builder().set_data_utf8(&value).build())),
            Ok(None) => Ok(None),
            Err(e) => Err(e.into()),
        }
    }

    async fn metadata_push(&self, _req: Payload) -> Result<()> {
        todo!()
    }

    async fn fire_and_forget(&self, _req: Payload) -> Result<()> {
        todo!()
    }

    fn request_stream(&self, _req: Payload) -> Flux<Result<Payload>> {
        todo!()
    }

    fn request_channel(&self, _reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        todo!()
    }
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
