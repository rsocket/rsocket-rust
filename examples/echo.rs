#[macro_use]
extern crate log;

use clap::{App, Arg, SubCommand};
use rsocket_rust::prelude::*;
use rsocket_rust::transport::Connection;
use rsocket_rust::Result;
use rsocket_rust_transport_tcp::{
    TcpClientTransport, TcpServerTransport, UnixClientTransport, UnixServerTransport,
};
use rsocket_rust_transport_websocket::{WebsocketClientTransport, WebsocketServerTransport};
use std::fs;

enum RequestMode {
    FNF,
    REQUEST,
    STREAM,
    CHANNEL,
}

async fn serve<A, B>(transport: A, mtu: usize) -> Result<()>
where
    A: Send + Sync + ServerTransport<Item = B> + 'static,
    B: Send + Sync + Transport + 'static,
{
    RSocketFactory::receive()
        .transport(transport)
        .fragment(mtu)
        .acceptor(Box::new(|setup, _socket| {
            info!("accept setup: {:?}", setup);
            Ok(Box::new(EchoRSocket))
            // Or you can reject setup
            // Err(From::from("SETUP_NOT_ALLOW"))
        }))
        .on_start(Box::new(|| info!("+++++++ echo server started! +++++++")))
        .serve()
        .await
}

async fn connect<A, B>(transport: A, mtu: usize, req: Payload, mode: RequestMode) -> Result<()>
where
    A: Send + Sync + Transport<Conn = B> + 'static,
    B: Send + Sync + Connection + 'static,
{
    let cli = RSocketFactory::connect()
        .fragment(mtu)
        .transport(transport)
        .start()
        .await?;

    match mode {
        RequestMode::FNF => {
            cli.fire_and_forget(req).await;
        }
        RequestMode::STREAM => {
            let mut results = cli.request_stream(req);
            loop {
                match results.next().await {
                    Some(Ok(v)) => info!("{:?}", v),
                    Some(Err(e)) => {
                        error!("STREAM_RESPONSE FAILED: {:?}", e);
                        break;
                    }
                    None => break,
                }
            }
        }
        RequestMode::CHANNEL => {
            let mut results = cli.request_channel(Box::pin(futures::stream::iter(vec![Ok(req)])));
            loop {
                match results.next().await {
                    Some(Ok(v)) => info!("{:?}", v),
                    Some(Err(e)) => {
                        error!("CHANNEL_RESPONSE FAILED: {:?}", e);
                        break;
                    }
                    None => break,
                }
            }
        }
        RequestMode::REQUEST => {
            let res = cli.request_response(req).await.expect("Request failed!");
            info!("{:?}", res);
        }
    }
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::builder().format_timestamp_millis().init();

    let cli = App::new("echo")
        .version("0.0.0")
        .author("Jeffsky <jjeffcaii@outlook.com>")
        .about("An echo tool for RSocket.")
        .subcommand(
            SubCommand::with_name("serve")
                .about("serve an echo server")
                .arg(
                    Arg::with_name("mtu")
                        .long("mtu")
                        .required(false)
                        .takes_value(true)
                        .help("Fragment mtu."),
                )
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
                    Arg::with_name("mtu")
                        .long("mtu")
                        .required(false)
                        .takes_value(true)
                        .help("Fragment mtu."),
                )
                .arg(
                    Arg::with_name("request")
                        .long("request")
                        .required(false)
                        .takes_value(false)
                        .help("request_response mode."),
                )
                .arg(
                    Arg::with_name("channel")
                        .long("channel")
                        .required(false)
                        .takes_value(false)
                        .help("request_channel mode."),
                )
                .arg(
                    Arg::with_name("stream")
                        .long("stream")
                        .required(false)
                        .takes_value(false)
                        .help("request_stream mode."),
                )
                .arg(
                    Arg::with_name("fnf")
                        .long("fnf")
                        .required(false)
                        .takes_value(false)
                        .help("fire_and_forget mode."),
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
            let mtu: usize = flags
                .value_of("mtu")
                .map(|it| it.parse().expect("Invalid mtu string!"))
                .unwrap_or(0);

            if addr.starts_with("ws://") {
                serve(WebsocketServerTransport::from(addr), mtu).await
            } else if addr.starts_with("unix://") {
                let addr_owned = addr.to_owned();
                tokio::spawn(async move {
                    let _ = serve(UnixServerTransport::from(addr_owned), mtu).await;
                });
                let sockfile = addr.chars().skip(7).collect::<String>();
                // Watch signal
                tokio::signal::ctrl_c().await?;
                info!("ctrl-c received!");
                if let Err(e) = std::fs::remove_file(&sockfile) {
                    error!("remove unix sock file failed: {}", e);
                }
                Ok(())
            } else {
                serve(TcpServerTransport::from(addr), mtu).await
            }
        }
        ("connect", Some(flags)) => {
            let mut modes: Vec<RequestMode> = vec![];

            if flags.is_present("stream") {
                modes.push(RequestMode::STREAM);
            }
            if flags.is_present("fnf") {
                modes.push(RequestMode::FNF);
            }
            if flags.is_present("channel") {
                modes.push(RequestMode::CHANNEL);
            }

            if flags.is_present("request") {
                modes.push(RequestMode::REQUEST);
            }

            if modes.len() > 1 {
                error!("duplicated request mode: use one of --fnf/--request/--stream/--channel.");
                return Ok(());
            }

            let mtu: usize = flags
                .value_of("mtu")
                .map(|it| it.parse().expect("Invalid mtu string!"))
                .unwrap_or(0);

            let addr = flags.value_of("URL").expect("Missing URL");
            let mut bu = Payload::builder();
            if let Some(data) = flags.value_of("input") {
                if data.starts_with("@") {
                    let file_content =
                        fs::read_to_string(&data[1..].to_owned()).expect("Read file failed.");
                    bu = bu.set_data_utf8(&file_content);
                } else {
                    bu = bu.set_data_utf8(data);
                }
            }
            let req = bu.build();
            let mode = modes.pop().unwrap_or(RequestMode::REQUEST);
            if addr.starts_with("ws://") {
                connect(WebsocketClientTransport::from(addr), mtu, req, mode).await
            } else if addr.starts_with("unix://") {
                connect(UnixClientTransport::from(addr), mtu, req, mode).await
            } else {
                connect(TcpClientTransport::from(addr), mtu, req, mode).await
            }
        }
        _ => Ok(()),
    }
}
