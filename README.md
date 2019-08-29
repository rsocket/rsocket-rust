# rsocket-rust
![logo](./logo.jpg)

[![License](https://img.shields.io/github/license/jjeffcaii/rsocket-rust.svg)](https://github.com/jjeffcaii/rsocket-rust/blob/master/LICENSE)
[![GitHub Release](https://img.shields.io/github/release-pre/jjeffcaii/rsocket-rust.svg)](https://github.com/jjeffcaii/rsocket-rust/releases)

> rsocket-rust is an implementation of the RSocket protocol in Rust.
<br>It is under active development. Do not use it in a production environment.

## Example

> Here's a prototype of RSocket Client API.

```rust
extern crate futures;
extern crate rsocket_rust;

use futures::prelude::*;
use rsocket_rust::prelude::*;

#[test]
fn test_client_request_response() {
  let cli = Client::builder()
    .set_uri("127.0.0.1:7878")
    .set_setup(Payload::from("READY!"))
    .set_data_mime_type("text/plain")
    .set_metadata_mime_type("text/plain")
    .build()
    .unwrap();
  let pa = Payload::builder()
    .set_data_utf8("Hello World!")
    .set_metadata_utf8("Rust!")
    .build();
  let resp = cli.request_response(pa).wait().unwrap();
  println!("====> response: {:?}", resp);
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
