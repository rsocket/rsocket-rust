use log::info;
use rsocket_rust::prelude::{ClientResponder, EchoRSocket, Payload, RSocket, RSocketFactory};
use rsocket_rust_transport_unix::UnixClientTransport;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    let responder: ClientResponder = Box::new(|| Box::new(EchoRSocket));

    let client = RSocketFactory::connect()
        .acceptor(responder)
        .transport(UnixClientTransport::from("/tmp/rsocket-uds.sock"))
        .setup(Payload::from("READY!"))
        .mime_type("text/plain", "text/plain")
        .start()
        .await
        .unwrap();

    let request_payload: Payload = Payload::builder()
        .set_data_utf8("Hello World!")
        .set_metadata_utf8("Rust")
        .build();

    let res = client.request_response(request_payload).await.unwrap();

    info!("got: {:?}", res);

    client.close();

    Ok(())
}
