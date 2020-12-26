use async_trait::async_trait;
use redis::Client as RedisClient;
use rsocket_rust::prelude::*;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::TcpServerTransport;
use std::str::FromStr;

#[derive(Clone)]
struct RedisDao {
    inner: RedisClient,
}

#[tokio::main]
async fn main() -> Result<()> {
    let dao = RedisDao::from_str("redis://127.0.0.1").expect("Connect redis failed!");
    RSocketFactory::receive()
        .acceptor(Box::new(move |_setup, _socket| Ok(Box::new(dao.clone()))))
        .transport(TcpServerTransport::from("127.0.0.1:7878"))
        .serve()
        .await
}

impl FromStr for RedisDao {
    type Err = redis::RedisError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        let client = redis::Client::open(s)?;
        Ok(RedisDao { inner: client })
    }
}

#[async_trait]
impl RSocket for RedisDao {
    async fn request_response(&self, req: Payload) -> Result<Payload> {
        let client = self.inner.clone();
        let mut conn: redis::aio::Connection = client
            .get_async_connection()
            .await
            .expect("Connect redis failed!");
        let value: String = redis::cmd("GET")
            .arg(&[req.data_utf8()])
            .query_async(&mut conn)
            .await
            .unwrap_or("<nil>".to_owned());
        Ok(Payload::builder().set_data_utf8(&value).build())
    }

    async fn metadata_push(&self, _req: Payload) -> Result<()> {
        unimplemented!()
    }

    async fn fire_and_forget(&self, _req: Payload) -> Result<()> {
        unimplemented!()
    }

    fn request_stream(&self, _req: Payload) -> Flux<Result<Payload>> {
        unimplemented!()
    }

    fn request_channel(&self, _reqs: Flux<Result<Payload>>) -> Flux<Result<Payload>> {
        unimplemented!()
    }
}
