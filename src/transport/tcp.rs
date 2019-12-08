use super::codec::RFrameCodec;
use super::misc;
use super::spi::{Rx, Tx};
use crate::frame::Frame;
use futures::{Sink, SinkExt, Stream, StreamExt};
use std::net::SocketAddr;
use std::net::TcpStream as StdTcpStream;
use tokio::net::TcpStream;
use tokio::prelude::*;
use tokio::sync::mpsc;
use tokio_util::codec::{Decoder, Encoder, Framed, FramedParts, FramedRead, FramedWrite};

pub fn connect(addr: &SocketAddr) -> TcpStream {
    let origin = StdTcpStream::connect(addr).unwrap();
    TcpStream::from_std(origin).unwrap()
}

pub async fn process(socket: TcpStream, mut inputs: Rx<Frame>, outputs: Tx<Frame>) {
    let (mut writer, mut reader) = Framed::new(socket, RFrameCodec).split();
    tokio::spawn(async move {
        // loop read
        loop {
            match reader.next().await {
                Some(it) => outputs.send(it.unwrap()).unwrap(),
                None => {
                    drop(outputs);
                    break;
                }
            }
        }
    });
    // loop write
    while let Some(it) = inputs.recv().await {
        misc::debug_frame(true, &it);
        writer.send(it).await.unwrap()
    }
}
