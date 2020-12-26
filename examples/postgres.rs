#[macro_use]
extern crate log;

use async_trait::async_trait;
use rsocket_rust::prelude::*;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::TcpServerTransport;
use std::sync::Arc;
use tokio_postgres::{Client as PgClient, NoTls};

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_timestamp_millis().init();
    let dao = Dao::try_new().await?;
    RSocketFactory::receive()
        .acceptor(Box::new(move |_, _| Ok(Box::new(dao.clone()))))
        .on_start(Box::new(|| info!("server start success!!!")))
        .transport(TcpServerTransport::from("127.0.0.1:7878"))
        .serve()
        .await
}

#[derive(Clone)]
struct Dao {
    client: Arc<PgClient>,
}

#[async_trait]
impl RSocket for Dao {
    async fn request_response(&self, _: Payload) -> Result<Payload> {
        let client = self.client.clone();
        let row = client
            .query_one("SELECT 'world' AS hello", &[])
            .await
            .expect("Execute SQL failed!");
        let result: String = row.get("hello");
        Ok(Payload::builder().set_data_utf8(&result).build())
    }

    async fn metadata_push(&self, _: Payload) -> Result<()> {
        unimplemented!()
    }

    async fn fire_and_forget(&self, _: Payload) -> Result<()> {
        unimplemented!()
    }

    fn request_stream(&self, _: Payload) -> Flux<Result<Payload>> {
        unimplemented!()
    }

    fn request_channel(&self, _: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        unimplemented!()
    }
}

impl Dao {
    async fn try_new() -> Result<Dao> {
        let (client, connection) =
            tokio_postgres::connect("host=localhost user=postgres password=postgres", NoTls)
                .await?;

        // The connection object performs the actual communication with the database,
        // so spawn it off to run on its own.
        tokio::spawn(async move {
            if let Err(e) = connection.await {
                eprintln!("connection error: {}", e);
            }
        });

        info!("==> create postgres pool success!");
        Ok(Dao {
            client: Arc::new(client),
        })
    }
}
