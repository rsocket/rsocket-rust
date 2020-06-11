use log::info;
use rsocket_rust::prelude::{EchoRSocket, RSocketFactory, ServerResponder};
use rsocket_rust_transport_unix::UnixServerTransport;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {

    let transport: UnixServerTransport = UnixServerTransport::from("/tmp/rsocket-uds.sock");

    let responder: ServerResponder = Box::new(|setup, _socket| {
        info!("accept setup: {:?}", setup);
        Ok(Box::new(EchoRSocket))
        // Or you can reject setup
        // Err(From::from("SETUP_NOT_ALLOW"))
    });

    let on_start: Box<dyn FnMut() + Send + Sync> =
        Box::new(|| info!("+++++++ echo server started! +++++++"));

    RSocketFactory::receive()
        .transport(transport)
        .acceptor(responder)
        .on_start(on_start)
        .serve()
        .await?;

    Ok(())
}