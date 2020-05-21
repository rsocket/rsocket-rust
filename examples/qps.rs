#[macro_use]
extern crate log;

use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::TcpClientTransport;
use std::error::Error;
use std::sync::{
    atomic::{AtomicU32, Ordering},
    Arc,
};
use std::time::SystemTime;
use tokio::runtime::Runtime;
use tokio::sync::Notify;

const TOTAL: u32 = 1_000_000;

fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::builder().format_timestamp_millis().init();

    let mut rt = Runtime::new()?;
    let client = rt.block_on(async {
        RSocketFactory::connect()
            .transport(TcpClientTransport::from("127.0.0.1:7878"))
            .start()
            .await
    })?;
    // simulate 1KB payload.
    let req = Payload::builder()
        .set_data_utf8("X".repeat(1024).as_ref())
        .build();
    let counter = Arc::new(AtomicU32::new(0));
    let start_time = SystemTime::now();
    let notify = Arc::new(Notify::new());
    for _ in 0..TOTAL {
        let client = client.clone();
        let counter = counter.clone();
        let notify = notify.clone();
        let req = req.clone();
        rt.spawn(async move {
            client.request_response(req).await.expect("Request failed");
            let current = counter.fetch_add(1, Ordering::SeqCst) + 1;
            if current >= TOTAL {
                notify.notify();
            }
        });
    }
    rt.block_on(async move {
        notify.notified().await;
    });
    let costs = SystemTime::now()
        .duration_since(start_time)
        .unwrap()
        .as_millis();
    info!(
        "total={}, cost={}ms, qps={}",
        TOTAL,
        costs,
        1000f64 * (TOTAL as f64) / (costs as f64)
    );
    Ok(())
}
