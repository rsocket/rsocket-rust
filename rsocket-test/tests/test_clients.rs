#[macro_use]
extern crate log;

use futures::stream;
use rsocket_rust::prelude::*;
use rsocket_rust::Client;
use rsocket_rust_transport_tcp::{
    TcpClientTransport, TcpServerTransport, UnixClientTransport, UnixServerTransport,
};
use rsocket_rust_transport_websocket::{WebsocketClientTransport, WebsocketServerTransport};
use std::thread::sleep;
use std::time::Duration;
use tokio::runtime::Runtime;

fn init() {
    let _ = env_logger::builder()
        .format_timestamp_millis()
        .is_test(true)
        .try_init();
}

#[tokio::main]
#[test]
async fn test_connect_must_failed() {
    let result = RSocketFactory::connect()
        .transport(TcpClientTransport::from("tcp://127.0.0.1:6789"))
        .start()
        .await;
    assert_eq!(false, result.is_ok());
}

#[test]
fn test_websocket() {
    init();

    let addr = "127.0.0.1:8080";

    let server_runtime = Runtime::new().unwrap();

    // spawn a server
    server_runtime.spawn(async move {
        RSocketFactory::receive()
            .transport(WebsocketServerTransport::from(addr))
            .acceptor(Box::new(|setup, _socket| {
                info!("accept setup: {:?}", setup);
                Ok(Box::new(EchoRSocket))
            }))
            .on_start(Box::new(|| {
                info!("+++++++ websocket echo server started! +++++++")
            }))
            .serve()
            .await
    });

    sleep(Duration::from_millis(500));

    let client_runtime = Runtime::new().unwrap();

    client_runtime.block_on(async {
        let cli = RSocketFactory::connect()
            .acceptor(Box::new(|| Box::new(EchoRSocket)))
            .transport(WebsocketClientTransport::from(addr))
            .setup(Payload::from("READY!"))
            .mime_type("text/plain", "text/plain")
            .start()
            .await
            .unwrap();

        info!("=====> begin");

        exec_metadata_push(&cli).await;
        exec_fire_and_forget(&cli).await;
        exec_request_response(&cli).await;
        exec_request_stream(&cli).await;
        exec_request_channel(&cli).await;
        cli.close();
    });
}

#[test]
fn test_tcp() {
    init();

    let addr = "127.0.0.1:7878";

    let server_runtime = Runtime::new().unwrap();

    // spawn a server
    server_runtime.spawn(async move {
        RSocketFactory::receive()
            .transport(TcpServerTransport::from(addr))
            .acceptor(Box::new(|setup, _socket| {
                info!("accept setup: {:?}", setup);
                Ok(Box::new(EchoRSocket))
            }))
            .on_start(Box::new(|| {
                info!("+++++++ tcp echo server started! +++++++")
            }))
            .serve()
            .await
    });

    sleep(Duration::from_millis(500));

    let client_runtime = Runtime::new().unwrap();

    client_runtime.block_on(async {
        let cli = RSocketFactory::connect()
            .acceptor(Box::new(|| Box::new(EchoRSocket)))
            .transport(TcpClientTransport::from(addr))
            .setup(Payload::from("READY!"))
            .mime_type("text/plain", "text/plain")
            .start()
            .await
            .unwrap();

        exec_metadata_push(&cli).await;
        exec_fire_and_forget(&cli).await;
        exec_request_response(&cli).await;
        exec_request_stream(&cli).await;
        exec_request_channel(&cli).await;
        cli.close();
    });
}

#[test]
fn test_unix() {
    init();

    let addr = "/tmp/rsocket-uds.sock";

    let server_runtime = Runtime::new().unwrap();

    // spawn a server
    server_runtime.spawn(async move {
        if let Err(e) = RSocketFactory::receive()
            .transport(UnixServerTransport::from(addr))
            .acceptor(Box::new(|setup, _socket| {
                info!("accept setup: {:?}", setup);
                Ok(Box::new(EchoRSocket))
            }))
            .on_start(Box::new(|| {
                info!("+++++++ unix echo server started! +++++++")
            }))
            .serve()
            .await
        {
            error!("server stopped with error: {}", e)
        }

        // Watch signal
        tokio::signal::ctrl_c().await.unwrap();
        info!("ctrl-c received!");
        if let Err(e) = std::fs::remove_file(addr) {
            error!("remove unix sock file failed: {}", e);
        }
    });

    sleep(Duration::from_millis(500));

    let client_runtime = Runtime::new().unwrap();

    client_runtime.block_on(async {
        let cli = RSocketFactory::connect()
            .acceptor(Box::new(|| Box::new(EchoRSocket)))
            .transport(UnixClientTransport::from(addr))
            .setup(Payload::from("READY!"))
            .mime_type("text/plain", "text/plain")
            .start()
            .await
            .unwrap();

        exec_metadata_push(&cli).await;
        exec_fire_and_forget(&cli).await;
        exec_request_response(&cli).await;
        exec_request_stream(&cli).await;
        exec_request_channel(&cli).await;
        cli.close();
    });
}

#[tokio::main]
#[test]
#[ignore]
async fn test_request_response_err() {
    env_logger::builder().format_timestamp_millis().init();

    let cli = RSocketFactory::connect()
        .transport(TcpClientTransport::from("127.0.0.1:7878"))
        .setup(Payload::from("READY!"))
        .mime_type("text/plain", "text/plain")
        .start()
        .await
        .unwrap();

    let res = cli
        .request_response(Payload::from("must return error"))
        .await;

    match res {
        Ok(_) => panic!("should catch an error!"),
        Err(e) => info!("error catched: {}", e),
    };
}

async fn exec_request_response(socket: &Client) {
    // request response
    let sending = Payload::builder()
        .set_data_utf8("Hello World!")
        .set_metadata_utf8("I Rust!")
        .build();
    let result = socket.request_response(sending).await.unwrap();
    info!("REQUEST_RESPONSE: {:?}", result);
}

async fn exec_metadata_push(socket: &Client) {
    let pa = Payload::builder().set_metadata_utf8("Hello World!").build();
    // metadata push
    socket.metadata_push(pa).await;
}

async fn exec_fire_and_forget(socket: &Client) {
    // request fnf
    let fnf = Payload::from("Hello World!");
    socket.fire_and_forget(fnf).await;
}

async fn exec_request_stream(socket: &Client) {
    // request stream
    let sending = Payload::builder()
        .set_data_utf8("Hello Rust!")
        .set_metadata_utf8("foobar")
        .build();

    let mut results = socket.request_stream(sending);
    loop {
        match results.next().await {
            Some(Ok(v)) => info!("STREAM_RESPONSE OK: {:?}", v),
            Some(Err(e)) => error!("STREAM_RESPONSE FAILED: {:?}", e),
            None => break,
        }
    }
}

async fn exec_request_channel(socket: &Client) {
    let sends: Vec<_> = (0..10)
        .map(|n| {
            let p = Payload::builder()
                .set_data_utf8(&format!("Hello#{}", n))
                .set_metadata_utf8("RUST")
                .build();
            Ok(p)
        })
        .collect();
    let mut results = socket.request_channel(Box::pin(stream::iter(sends)));
    loop {
        match results.next().await {
            Some(Ok(v)) => info!("CHANNEL_RESPONSE OK: {:?}", v),
            Some(Err(e)) => error!("CHANNEL_RESPONSE FAILED: {:?}", e),
            None => break,
        }
    }
}
