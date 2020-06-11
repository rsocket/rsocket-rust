use super::client::UnixClientTransport;
use rsocket_rust::transport::{ClientTransport, ServerTransport};
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use tokio::net::UnixListener;

pub struct UnixServerTransport {
    addr: String,
}

impl UnixServerTransport {
    fn new(addr: String) -> UnixServerTransport {
        UnixServerTransport { addr }
    }
}

impl ServerTransport for UnixServerTransport {
    type Item = UnixClientTransport;

    fn start(
        self,
        starter: Option<Box<dyn FnMut() + Send + Sync>>,
        acceptor: impl Fn(Self::Item) + Send + Sync + 'static,
    ) -> Pin<Box<dyn Send + Future<Output = Result<(), Box<dyn Send + Sync + Error>>>>>
    where
        Self::Item: ClientTransport + Sized,
    {
        Box::pin(async move {
            match UnixListener::bind(&self.addr.as_str()){
                Ok(mut listener) => {
                    debug!("listening on: {}", &self.addr);
                    if let Some(mut bingo) = starter {
                        bingo();
                    }
                    while let Ok((socket, _)) = listener.accept().await {
                        let tp = UnixClientTransport::from(socket);
                        acceptor(tp);
                    }
                    
                    Ok(())
                }
                Err(e) => Err(e.into_inner().unwrap()),
            }
        })
    }
}

impl Drop for UnixServerTransport {
    fn drop(&mut self) {
        std::fs::remove_file(&self.addr.as_str()).unwrap();
    }
}
// impl From<SocketAddr> for UnixServerTransport {
//     fn from(addr: SocketAddr) -> UnixServerTransport {
//         UnixServerTransport::new(addr)
//     }
// }

impl From<String> for UnixServerTransport {
    fn from(addr: String) -> UnixServerTransport {
        UnixServerTransport::new(addr.parse().unwrap())
    }
}

impl From<&str> for UnixServerTransport {
    fn from(addr: &str) -> UnixServerTransport {
        UnixServerTransport::new(addr.parse().unwrap())
    }
}
