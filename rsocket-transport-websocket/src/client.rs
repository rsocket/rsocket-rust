use bytes::{BufMut, BytesMut};
use futures::{SinkExt, StreamExt};
use rsocket_rust::frame::{Frame, Writeable};
use rsocket_rust::transport::{BoxResult, ClientTransport, Rx, SafeFuture, Tx};
use std::error::Error;
use std::net::SocketAddr;
use tokio::net::TcpStream;
use tokio_tungstenite::{accept_async, connect_async, tungstenite::Message, WebSocketStream};
use url::Url;

enum Connector {
    Direct(TcpStream),
    Lazy(Url),
}

pub struct WebsocketClientTransport {
    connector: Connector,
}

impl WebsocketClientTransport {
    fn new(connector: Connector) -> WebsocketClientTransport {
        WebsocketClientTransport { connector }
    }

    async fn connect(self) -> Result<WebSocketStream<TcpStream>, Box<dyn Send + Sync + Error>> {
        match self.connector {
            Connector::Direct(stream) => match accept_async(stream).await {
                Ok(ws) => Ok(ws),
                Err(e) => Err(Box::new(e)),
            },
            Connector::Lazy(u) => match connect_async(u).await {
                Ok((stream, _)) => Ok(stream),
                Err(e) => Err(Box::new(e)),
            },
        }
    }
}

impl ClientTransport for WebsocketClientTransport {
    fn attach(self, incoming: Tx<Frame>, mut sending: Rx<Frame>) -> SafeFuture<BoxResult<()>> {
        Box::pin(async move {
            let ws_stream = self.connect().await?;
            let (mut write, mut read) = ws_stream.split();
            tokio::spawn(async move {
                while let Some(next) = read.next().await {
                    match next {
                        Ok(msg) => {
                            let raw = msg.into_data();
                            let mut bf = BytesMut::new();
                            bf.put_slice(&raw[..]);
                            let f = Frame::decode(&mut bf).unwrap();
                            incoming.send(f).unwrap();
                        }
                        Err(e) => error!("got error: {}", e),
                    }
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

impl From<TcpStream> for WebsocketClientTransport {
    fn from(socket: TcpStream) -> WebsocketClientTransport {
        WebsocketClientTransport::new(Connector::Direct(socket))
    }
}

impl From<&str> for WebsocketClientTransport {
    fn from(addr: &str) -> WebsocketClientTransport {
        let u = if addr.starts_with("ws://") {
            Url::parse(addr).unwrap()
        } else {
            Url::parse(&format!("ws://{}", addr)).unwrap()
        };
        WebsocketClientTransport::new(Connector::Lazy(u))
    }
}

impl From<SocketAddr> for WebsocketClientTransport {
    fn from(addr: SocketAddr) -> WebsocketClientTransport {
        let u = Url::parse(&format!("ws://{}", addr)).unwrap();
        WebsocketClientTransport::new(Connector::Lazy(u))
    }
}

impl From<Url> for WebsocketClientTransport {
    fn from(url: Url) -> WebsocketClientTransport {
        WebsocketClientTransport::new(Connector::Lazy(url))
    }
}
