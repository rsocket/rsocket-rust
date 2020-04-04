use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpClientTransport;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
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
    let res = client.request_response(req).await.expect("Requet failed!");
    println!("request success: response={:?}", res);

    Ok(())
}
