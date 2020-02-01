use bytes::{BufMut, BytesMut};
use futures::{SinkExt, StreamExt};
use rsocket_rust::frame::{Frame, Writeable};
use rsocket_rust::transport::{ClientTransport, Rx, Tx};
use std::error::Error;
use std::future::Future;
use std::net::{SocketAddr, TcpStream as StdTcpStream};
use std::pin::Pin;
use tokio::net::TcpStream;
use tokio_tungstenite::tungstenite::Message;
use url::Url;

pub struct WebsocketClientTransport {
    socket: TcpStream,
}

impl From<TcpStream> for WebsocketClientTransport {
    fn from(socket: TcpStream) -> WebsocketClientTransport {
        WebsocketClientTransport { socket }
    }
}

impl From<&str> for WebsocketClientTransport {
    fn from(addr: &str) -> WebsocketClientTransport {
        let socket_addr = addr.parse().unwrap();
        WebsocketClientTransport {
            socket: connect(&socket_addr),
        }
    }
}

impl From<Url> for WebsocketClientTransport {
    fn from(addr: Url) -> WebsocketClientTransport {
        unimplemented!()
    }
}

impl ClientTransport for WebsocketClientTransport {
    fn attach(
        self,
        incoming: Tx<Frame>,
        mut sending: Rx<Frame>,
    ) -> Pin<Box<dyn Sync + Send + Future<Output = Result<(), Box<dyn Error + Send + Sync>>>>> {
        Box::pin(async move {
            let ws_stream = tokio_tungstenite::accept_async(self.socket)
                .await
                .expect("Error during the websocket handshake occurred");
            let (mut write, mut read) = ws_stream.split();
            tokio::spawn(async move {
                while let Some(Ok(msg)) = read.next().await {
                    let raw = msg.into_data();
                    let mut bf = BytesMut::new();
                    bf.put_slice(&raw[..]);
                    let f = Frame::decode(&mut bf).unwrap();
                    incoming.send(f).unwrap();
                }
            });
            while let Some(it) = sending.recv().await {
                debug!("===> SND: {:?}", &it);
                let mut bf = BytesMut::new();
                it.write_to(&mut bf);
                let msg = Message::binary(bf.to_vec());
                write.send(msg).await.unwrap();
            }
            Ok(())
        })
    }
}

#[inline]
fn connect(addr: &SocketAddr) -> TcpStream {
    let origin = StdTcpStream::connect(addr).unwrap();
    TcpStream::from_std(origin).unwrap()
}
