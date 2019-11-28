extern crate rsocket_rust;
extern crate tokio;
#[macro_use]
extern crate log;
use rsocket_rust::prelude::*;

#[tokio::main]
async fn main() {
    env_logger::builder().init();
    let setup = Payload::builder()
        .set_data_utf8("Hello")
        .set_metadata_utf8("World")
        .build();
    let cli = RSocketFactory::connect()
        .transport(URI::Tcp("127.0.0.1:7878".to_string()))
        .setup(setup)
        .start()
        .await;

    std::thread::sleep_ms(500000);
}
