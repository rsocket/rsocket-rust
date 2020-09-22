use crate::client::UnixClientTransport;
use crate::misc::parse_uds_addr;
use async_trait::async_trait;
use rsocket_rust::{transport::ServerTransport, Result};
use tokio::net::UnixListener;

#[derive(Debug)]
pub struct UnixServerTransport {
    addr: String,
    listener: Option<UnixListener>,
}

impl UnixServerTransport {
    fn new(addr: String) -> UnixServerTransport {
        UnixServerTransport {
            addr,
            listener: None,
        }
    }
}

#[async_trait]
impl ServerTransport for UnixServerTransport {
    type Item = UnixClientTransport;

    async fn start(&mut self) -> Result<()> {
        if self.listener.is_some() {
            return Ok(());
        }
        match UnixListener::bind(&self.addr) {
            Ok(listener) => {
                self.listener = Some(listener);
                debug!("listening on: {}", &self.addr);
                Ok(())
            }
            Err(e) => Err(Box::new(e)),
        }
    }

    async fn next(&mut self) -> Option<Result<Self::Item>> {
        match self.listener.as_mut() {
            Some(listener) => match listener.accept().await {
                Ok((socket, _)) => Some(Ok(UnixClientTransport::from(socket))),
                Err(e) => Some(Err(Box::new(e))),
            },
            None => None,
        }
    }
}

impl Drop for UnixServerTransport {
    fn drop(&mut self) {
        if let Err(e) = std::fs::remove_file(&self.addr) {
            warn!("remove unix sock file failed: {}", e);
        }
    }
}

impl From<String> for UnixServerTransport {
    fn from(addr: String) -> UnixServerTransport {
        UnixServerTransport::new(parse_uds_addr(addr))
    }
}

impl From<&str> for UnixServerTransport {
    fn from(addr: &str) -> UnixServerTransport {
        UnixServerTransport::new(parse_uds_addr(addr))
    }
}
