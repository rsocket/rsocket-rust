#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use rsocket_rust::prelude::*;
use rsocket_rust_messaging::*;
use rsocket_rust_transport_tcp::TcpClientTransport;

#[derive(Serialize, Deserialize, Debug)]
pub struct Student {
    id: i64,
    name: String,
    birth: String,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Response<T> {
    code: i32,
    message: Option<String>,
    data: T,
}

#[tokio::main]
#[test]
#[ignore]
async fn test_messaging() {
    let rsocket = RSocketFactory::connect()
        .transport(TcpClientTransport::from("tcp://127.0.0.1:7878"))
        .data_mime_type("application/json")
        .metadata_mime_type("message/x.rsocket.composite-metadata.v0")
        .start()
        .await
        .expect("Connect failed!");
    let requester = Requester::from(rsocket);

    let post = Student {
        id: 1234,
        name: "Jeffsky".to_owned(),
        birth: "2020-01-01".to_owned(),
    };
    let mut req = requester.route("student.v1.upsert");
    req.metadata_raw("raw metadata", "application/json")
        .unwrap();
    req.metadata(&post, "application/json").unwrap();
    req.data(&post).unwrap();
    let res: Response<Student> = req
        .retrieve_mono()
        .await
        .block()
        .expect("Retrieve failed!")
        .expect("Empty result!");
    println!("------> RESPONSE: {:?}", res);
}

#[test]
fn test_builder() {
    RequesterBuilder::default().data_mime_type("application/json");
}
