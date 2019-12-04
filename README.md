# rsocket-rust

[![Crates.io](https://img.shields.io/crates/v/rsocket_rust)](https://crates.io/crates/rsocket_rust)
[![Crates.io](https://img.shields.io/crates/d/rsocket_rust)](https://crates.io/crates/rsocket_rust)
[![License](https://img.shields.io/github/license/rsocket/rsocket-rust.svg)](https://github.com/rsocket/rsocket-rust/blob/master/LICENSE)
[![GitHub Release](https://img.shields.io/github/release-pre/rsocket/rsocket-rust.svg)](https://github.com/rsocket/rsocket-rust/releases)

> rsocket-rust is an implementation of the RSocket protocol in Rust(1.39+).
It's an **alpha** version and still under active development.  
**Do not use it in a production environment!**  

## Example

> Here are some example codes which show how RSocket works in Rust.

### Server

```rust
extern crate rsocket_rust;
extern crate tokio;
#[macro_use]
extern crate log;
use rsocket_rust::prelude::*;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    env_logger::builder().init();
    let addr = env::args().nth(1).unwrap_or("tcp://127.0.0.1:7878".to_string());

    RSocketFactory::receive()
        .transport(&addr)
        .acceptor(|setup, _socket| {
            info!("accept setup: {:?}", setup);
            Box::new(EchoRSocket)
        })
        .serve()
        .await
}
```

### Client

```rust
extern crate rsocket_rust;

use rsocket_rust::prelude::*;

#[tokio::main]
#[test]
async fn test() {
    let cli = RSocketFactory::connect()
        .acceptor(|| Box::new(EchoRSocket))
        .transport("tcp://127.0.0.1:7878")
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

## Dependencies

- [tokio](https://tokio.rs/)
- [futures-rs](http://rust-lang-nursery.github.io/futures-rs/)

## TODO

- Operations
  - [x] METADATA_PUSH
  - [x] REQUEST_FNF
  - [x] REQUEST_RESPONSE
  - [x] REQUEST_STREAM
  - [x] REQUEST_CHANNEL
- More Operations
  - [ ] Error
  - [ ] Cancel
  - [ ] Fragmentation
  - [ ] Resume
- QoS
  - [ ] RequestN
  - [ ] Lease
- Transport
  - [x] TCP
  - [ ] Websocket
- Reactor
  - [ ] ...
- High Level APIs
  - [x] Client
  - [x] Server
