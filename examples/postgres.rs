#[macro_use]
extern crate log;

use rsocket_rust::{error::RSocketError, prelude::*};
use rsocket_rust_transport_tcp::TcpServerTransport;
use std::error::Error;
use std::sync::Arc;
use tokio_postgres::{Client as PgClient, NoTls};

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
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

impl RSocket for Dao {
    fn request_response(&self, _: Payload) -> Mono<Result<Payload, RSocketError>> {
        let client = self.client.clone();
        Box::pin(async move {
            let row = client
                .query_one("SELECT 'world' AS hello", &[])
                .await
                .expect("Execute SQL failed!");
            let result: String = row.get("hello");
            Ok(Payload::builder().set_data_utf8(&result).build())
        })
    }

    fn metadata_push(&self, _: Payload) -> Mono<()> {
        unimplemented!()
    }

    fn fire_and_forget(&self, _: Payload) -> Mono<()> {
        unimplemented!()
    }

    fn request_stream(&self, _: Payload) -> Flux<Result<Payload, RSocketError>> {
        unimplemented!()
    }

    fn request_channel(
        &self,
        _: Flux<Result<Payload, RSocketError>>,
    ) -> Flux<Result<Payload, RSocketError>> {
        unimplemented!()
    }
}

impl Dao {
    async fn try_new() -> Result<Dao, Box<dyn Error + Sync + Send>> {
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
