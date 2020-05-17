#[macro_use]
extern crate log;
#[macro_use]
extern crate serde_derive;

use rsocket_rust::extension::MimeType;
use rsocket_rust_messaging::*;

fn init() {
    let _ = env_logger::builder()
        .format_timestamp_millis()
        .is_test(true)
        .try_init();
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Token {
    app: String,
    access: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Student {
    id: i64,
    name: String,
    birth: String,
}

#[derive(Serialize, Deserialize, Debug, Default)]
pub struct Tracing {
    id: String,
    ts: i64,
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
    init();
    let token = Token {
        app: "xxx".to_owned(),
        access: "yyy".to_owned(),
    };
    let requester = Requester::builder()
        .setup_metadata(&token, MimeType::APPLICATION_JSON)
        .setup_data(&token)
        .connect_tcp("127.0.0.1", 7878)
        .build()
        .await
        .expect("Connect failed!");

    let post = Student {
        id: 1234,
        name: "Jeffsky".to_owned(),
        birth: "2020-01-01".to_owned(),
    };

    let res: Response<Student> = requester
        .route("student.v1.upsert")
        .metadata(Tracing::default(), "application/json")
        .metadata_raw("foobar", "message/x.rsocket.authentication.bearer.v0")
        .data(post)
        .retrieve_mono()
        .await
        .block()
        .expect("Retrieve failed!")
        .expect("Empty result!");
    info!("------> RESPONSE: {:?}", res);
}
