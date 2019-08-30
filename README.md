# rsocket-rust
![logo](./logo.jpg)

[![Crates.io](https://img.shields.io/crates/v/rsocket_rust)](https://crates.io/crates/rsocket_rust)
[![Crates.io](https://img.shields.io/crates/d/rsocket_rust)](https://crates.io/crates/rsocket_rust)
[![License](https://img.shields.io/github/license/jjeffcaii/rsocket-rust.svg)](https://github.com/jjeffcaii/rsocket-rust/blob/master/LICENSE)
[![GitHub Release](https://img.shields.io/github/release-pre/jjeffcaii/rsocket-rust.svg)](https://github.com/jjeffcaii/rsocket-rust/releases)

> rsocket-rust is an implementation of the RSocket protocol in Rust.
<br>It is under active development. Do not use it in a production environment.

## Example

> Here are some example codes which show how RSocket works in Rust. :sunglasses:

### Server
```rust
extern crate futures;
extern crate rsocket_rust;

use futures::prelude::*;
use rsocket_rust::prelude::*;

#[test]
fn test_serve() {
  RSocketFactory::receive()
    .transport(URI::Tcp("127.0.0.1:7878"))
    .acceptor(Box::new(MockResponder))
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
    .acceptor(Box::new(MockResponder))
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

## TODO
 - Codec
   - [x] Setup
   - [x] Keepalive
   - [x] Payload
   - [x] RequestResponse
   - [x] RequestStream
   - [x] RequestChannel
   - [x] RequestFireAndForget
   - [x] MetadataPush
   - [x] RequestN
   - [ ] Resume
   - [x] ResumeOK
   - [x] Cancel
   - [x] Error
   - [x] Lease
 - Transport
   - [ ] TCP
   - [ ] Websocket
 - Rx
   - [ ] ...
 - High Level APIs
   - [ ] Client
   - [ ] Server
