use bytes::{BufMut, BytesMut};
use futures::{SinkExt, StreamExt};
use rsocket_rust::frame::{Frame, Writeable};
use rsocket_rust::transport::{ClientTransport, Rx, Tx};
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use tokio_tungstenite::connect_async;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

pub struct WebsocketClientTransport {
    addr: Url,
}

impl From<&str> for WebsocketClientTransport {
    fn from(addr: &str) -> WebsocketClientTransport {
        WebsocketClientTransport {
            addr: Url::parse(addr).unwrap(),
        }
    }
}

impl From<Url> for WebsocketClientTransport {
    fn from(addr: Url) -> WebsocketClientTransport {
        WebsocketClientTransport { addr }
    }
}

impl ClientTransport for WebsocketClientTransport {
    fn attach(
        self,
        incoming: Tx<Frame>,
        mut sending: Rx<Frame>,
    ) -> Pin<Box<dyn Sync + Send + Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>> {
        Box::pin(async move {
            let (ws_stream, _) = connect_async(self.addr).await.expect("Failed to connect");
            let (mut write, mut read) = ws_stream.split();
            tokio::spawn(async move {
                while let Some(msg) = read.next().await {
                    let raw = msg.unwrap().into_data();
                    let mut bf = BytesMut::new();
                    bf.put_slice(&raw[..]);
                    let f = Frame::decode(&mut bf).unwrap();
                    incoming.send(f).unwrap();
                }
            });
            while let Some(it) = sending.recv().await {
                let mut bf = BytesMut::new();
                it.write_to(&mut bf);
                let msg = Message::binary(bf.to_vec());
                write.send(msg).await.unwrap();
            }
            Ok(())
        })
    }
}
