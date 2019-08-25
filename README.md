# rsocket-rust
learn rust && implement rsocket. 😭

## Example

> Here's a prototype of RSocket Client API.

```rust
extern crate bytes;
extern crate futures;
extern crate rsocket;
extern crate tokio;

use bytes::Bytes;
use futures::{Future, Stream};
use rsocket::prelude::*;

#[test]
fn test_socket_request() {
  // Prepare a Echo server. (see: https://github.com/rsocket/rsocket-go/tree/master/cmd/echo)

  // create a socket.
  let socket = DuplexSocket::builder("127.0.0.1:7878")
    .set_setup(Payload::from("Ready!"))
    .connect();

  // request fnf
  let fnf = Payload::from("Mock FNF");
  socket.request_fnf(fnf).wait().unwrap();

  // request response
  for n in 0..10 {
    let sending = Payload::builder()
      .set_data(Bytes::from(format!("[{}] Hello Rust!", n)))
      .set_metadata(Bytes::from("text/plain"))
      .build();
    let result = socket.request_response(sending).wait().unwrap();
    println!("********** YES: {:?}", result);
  }

  // request stream
  let sending = Payload::builder()
    .set_data(Bytes::from("Hello Rust!"))
    .set_metadata(Bytes::from("text/plain"))
    .build();
  let task = socket
    .request_stream(sending)
    .map_err(|_| ())
    .for_each(|it| {
      println!("******* STREAM: {:?}", it);
      Ok(())
    });
  task.wait().unwrap();
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
