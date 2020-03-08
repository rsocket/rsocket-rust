#[macro_use]
extern crate log;

use clap::{App, Arg, SubCommand};
use rsocket_rust::prelude::*;
use rsocket_rust_transport_tcp::{TcpClientTransport, TcpServerTransport};
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error + Send + Sync>> {
    env_logger::builder().format_timestamp_millis().init();

    let cli = App::new("echo")
        .version("0.0.0")
        .author("Jeffsky <jjeffcaii@outlook.com>")
        .about("An echo tool for RSocket.")
        .subcommand(
            SubCommand::with_name("serve")
                .about("serve an echo server")
                .arg(
                    Arg::with_name("URL")
                        .required(true)
                        .index(1)
                        .help("connect url"),
                ),
        )
        .subcommand(
            SubCommand::with_name("connect")
                .about("connect to echo server")
                .arg(
                    Arg::with_name("input")
                        .short("i")
                        .long("input")
                        .required(false)
                        .takes_value(true)
                        .help("Input payload data."),
                )
                .arg(
                    Arg::with_name("URL")
                        .required(true)
                        .index(1)
                        .help("connect url"),
                ),
        )
        .arg(
            Arg::with_name("debug")
                .short("d")
                .help("print debug information verbosely"),
        )
        .get_matches();

    match cli.subcommand() {
        ("serve", Some(flags)) => {
            let addr = flags.value_of("URL").expect("Missing URL");
            RSocketFactory::receive()
                .transport(TcpServerTransport::from(addr))
                .acceptor(|setup, _socket| {
                    info!("accept setup: {:?}", setup);
                    Ok(Box::new(EchoRSocket))
                    // Or you can reject setup
                    // Err(From::from("SETUP_NOT_ALLOW"))
                })
                .on_start(Box::new(|| info!("+++++++ echo server started! +++++++")))
                .serve()
                .await
        }
        ("connect", Some(flags)) => {
            let addr = flags.value_of("URL").expect("Missing URL");
            let cli = RSocketFactory::connect()
                .transport(TcpClientTransport::from(addr))
                .start()
                .await
                .expect("Connect failed!");
            let mut bu = Payload::builder();
            if let Some(data) = flags.value_of("input") {
                bu = bu.set_data_utf8(data);
            }
            let req = bu.build();
            let res = cli.request_response(req).await.expect("Request failed!");
            info!("request success: {:?}", res);
            Ok(())
        }
        _ => Ok(()),
    }
}
