#[macro_use]
extern crate log;

use postgres::NoTls;
use r2d2_postgres::{
    r2d2::{self, Pool},
    PostgresConnectionManager,
};
use rsocket_rust::{error::RSocketError, prelude::*};
use rsocket_rust_transport_tcp::TcpServerTransport;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::builder().format_timestamp_millis().init();
    let dao = Dao::try_new().expect("Connect failed!");
    RSocketFactory::receive()
        .acceptor(Box::new(move |_, _| {
            info!("accept new socket!");
            Ok(Box::new(dao.clone()))
        }))
        .on_start(Box::new(|| info!("server start success!!!")))
        .transport(TcpServerTransport::from("127.0.0.1:7878"))
        .serve()
        .await
}

#[derive(Clone, Debug)]
struct Dao {
    pool: Pool<PostgresConnectionManager<NoTls>>,
}

impl RSocket for Dao {
    fn request_response(&self, _: Payload) -> Mono<Result<Payload, RSocketError>> {
        let pool = self.pool.clone();
        Box::pin(async move {
            // TODO: something wrong here!!!
            let mut client = pool.get().expect("Get client from pool failed!");
            let row = client
                .query_one("SELECT 'world' AS hello", &[])
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
    fn try_new() -> Result<Dao, Box<dyn Error + Sync + Send>> {
        let manager: PostgresConnectionManager<NoTls> = PostgresConnectionManager::new(
            "host=localhost user=postgres password=postgres"
                .parse()
                .unwrap(),
            NoTls,
        );
        let pool: Pool<PostgresConnectionManager<NoTls>> =
            r2d2::Pool::new(manager).expect("Create pool failed!");
        info!("==> create postgres pool success!");
        Ok(Dao { pool })
    }
}
