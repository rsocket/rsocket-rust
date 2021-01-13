#[macro_use]
extern crate log;

use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::time::SystemTime;

use clap::{App, Arg};
use rsocket_rust::prelude::*;
use rsocket_rust::transport::{Connection, Transport};
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::{TcpClientTransport, UnixClientTransport};
use rsocket_rust_transport_websocket::WebsocketClientTransport;
use tokio::sync::{oneshot, Notify};

async fn connect<A, B>(
    started: oneshot::Sender<SystemTime>,
    transport: A,
    count: u32,
    payload_size: usize,
    notify: Arc<Notify>,
) -> Result<()>
where
    A: Send + Sync + Transport<Conn = B> + 'static,
    B: Send + Sync + Connection + 'static,
{
    let client = RSocketFactory::connect()
        .transport(transport)
        .start()
        .await?;

    // simulate customize size payload.
    let req = Payload::builder()
        .set_data_utf8("_".repeat(payload_size).as_ref())
        .build();
    let start_time = SystemTime::now();
    let counter = Arc::new(AtomicU32::new(0));

    for _ in 0..count {
        let client = client.clone();
        let counter = counter.clone();
        let notify = notify.clone();
        let req = req.clone();
        tokio::spawn(async move {
            if let Err(e) = client.request_response(req).await {
                error!("request failed: {}", e);
            }
            let current = counter.fetch_add(1, Ordering::SeqCst) + 1;
            if current >= count {
                notify.notify_one();
            }
        });
    }
    started.send(start_time).unwrap();
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_timestamp_millis().init();

    let cli = App::new("echo")
        .version("0.0.0")
        .author("Jeffsky <jjeffcaii@outlook.com>")
        .about("An QPS benchmark tool for RSocket.")
        .arg(
            Arg::with_name("count")
                .short("c")
                .long("count")
                .required(false)
                .takes_value(true)
                .default_value("1000000")
                .help("Requests counts."),
        )
        .arg(
            Arg::with_name("size")
                .short("s")
                .long("size")
                .required(false)
                .takes_value(true)
                .default_value("1024")
                .help("Data size."),
        )
        .arg(
            Arg::with_name("URL")
                .required(false)
                .index(1)
                .default_value("127.0.0.1:7878")
                .help("Connect url"),
        )
        .get_matches();

    let count: u32 = cli
        .value_of("count")
        .map(|s| s.parse().expect("Invalid count!"))
        .unwrap();
    let size: usize = cli
        .value_of("size")
        .map(|s| s.parse().expect("Invalid size!"))
        .unwrap();
    let addr = cli.value_of("URL").unwrap();

    let notify = Arc::new(Notify::new());
    let (started_tx, started_rx) = oneshot::channel::<SystemTime>();

    if addr.starts_with("ws://") {
        connect(
            started_tx,
            WebsocketClientTransport::from(addr),
            count,
            size,
            notify.clone(),
        )
        .await?;
    } else if addr.starts_with("unix://") {
        connect(
            started_tx,
            UnixClientTransport::from(addr),
            count,
            size,
            notify.clone(),
        )
        .await?;
    } else {
        connect(
            started_tx,
            TcpClientTransport::from(addr),
            count,
            size,
            notify.clone(),
        )
        .await?;
    }
    notify.notified().await;
    let start_time = started_rx.await.unwrap();
    let costs = SystemTime::now()
        .duration_since(start_time)
        .unwrap()
        .as_millis();
    info!(
        "total={}, cost={}ms, qps={}",
        count,
        costs,
        1000f64 * (count as f64) / (costs as f64)
    );
    Ok(())
}
