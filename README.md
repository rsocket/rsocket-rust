# rsocket-rust

[![Crates.io](https://img.shields.io/crates/v/rsocket_rust)](https://crates.io/crates/rsocket_rust)
[![Crates.io](https://img.shields.io/crates/d/rsocket_rust)](https://crates.io/crates/rsocket_rust)
[![License](https://img.shields.io/github/license/rsocket/rsocket-rust.svg)](https://github.com/rsocket/rsocket-rust/blob/master/LICENSE)
[![GitHub Release](https://img.shields.io/github/release-pre/rsocket/rsocket-rust.svg)](https://github.com/rsocket/rsocket-rust/releases)

> rsocket-rust is an implementation of the RSocket protocol in Rust.
It's an **alpha** version and still under active development. **Do not use it in a production environment!**

## Example

> Here are some example codes which show how RSocket works in Rust.

### Server

```rust
extern crate bytes;
extern crate futures;
extern crate rsocket_rust;

use bytes::Bytes;
use futures::prelude::*;
use rsocket_rust::prelude::*;

#[test]
fn test_serve() {
  RSocketFactory::receive()
    .transport(URI::Tcp("127.0.0.1:7878"))
    .acceptor(|setup, sending_socket| {
      println!("accept setup: {:?}", setup);
      // TODO: use tokio runtime?
      std::thread::spawn(move || {
        let resp = sending_socket
          .request_response(
            Payload::builder()
              .set_data(Bytes::from("Hello Client!"))
              .build(),
          )
          .wait()
          .unwrap();
        println!(">>>>> response success: {:?}", resp);
      });
      Box::new(MockResponder)
    })
    .serve()
    .wait()
    .unwrap();
}

```

### Client

```rust
extern crate futures;
extern crate rsocket_rust;

use futures::prelude::*;
use rsocket_rust::prelude::*;

#[test]
fn test_client() {
  let cli = RSocketFactory::connect()
    .acceptor(||Box::new(MockResponder))
    .transport(URI::Tcp("127.0.0.1:7878"))
    .setup(Payload::from("READY!"))
    .mime_type("text/plain", "text/plain")
    .start()
    .unwrap();
  let pa = Payload::builder()
    .set_data_utf8("Hello World!")
    .set_metadata_utf8("Rust!")
    .build();
  let resp = cli.request_response(pa).wait().unwrap();
  println!("******* response: {:?}", resp);
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
  - [ ] REQUEST_CHANNEL
- Transport
  - [x] TCP
  - [ ] Websocket
- Reactor
  - [ ] ...
- High Level APIs
  - [ ] Client
  - [ ] Server
