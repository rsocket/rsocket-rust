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
            Ok(Box::new(EchoRSocket))
            // Or you can reject setup
            // Err(From::from("SETUP_NOT_ALLOW"))
        }))
        .on_start(Box::new(|| println!("+++++++ echo server started! +++++++")))
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
        .on_close(Box::new(|| println!("connection closed")))
        .start()
        .await?;
    let req = Payload::builder()
        .set_data_utf8("Hello World!")
        .set_metadata_utf8("Rust")
        .build();
    let res = cli.request_response(req).await?;
    println!("got: {:?}", res);

    // If you want to block until socket disconnected.
    cli.wait_for_close().await;

    Ok(())
}
```

### Implement RSocket trait

Example for access Redis([crates](https://crates.io/crates/redis)):

> NOTICE: add dependency in Cargo.toml => redis = { version = "0.19.0", features = [ "aio" ] }

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

