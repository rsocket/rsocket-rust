use crate::errors::RSocketError;
use std::error::Error;
use std::net::SocketAddr;
use std::result::Result;
use url::Url;

const SCHEMA_TCP: &str = "tcp";
const SCHEMA_WS: &str = "ws";
const DEFAULT_TCP_PORT: u16 = 7878;

#[derive(Debug, Clone)]
pub(crate) enum URI {
    Tcp(SocketAddr),
    Websocket(String),
}

impl URI {
    pub(crate) fn parse(s: &str) -> Result<URI, Box<dyn Error>> {
        match Url::parse(s) {
            Ok(u) => Self::from_url(u),
            Err(e) => Err(Box::new(e)),
        }
    }

    #[inline]
    fn from_url(u: Url) -> Result<URI, Box<dyn Error>> {
        let domain = u.domain().unwrap_or("0.0.0.0");
        let schema = u.scheme();
        match schema.to_lowercase().as_ref() {
            SCHEMA_TCP => {
                match format!("{}:{}", domain, u.port().unwrap_or(DEFAULT_TCP_PORT)).parse() {
                    Ok(addr) => Ok(URI::Tcp(addr)),
                    Err(e) => Err(Box::new(e)),
                }
            }
            _ => {
                let e = RSocketError::from(format!("invalid schema: {}", schema));
                Err(Box::new(e))
            }
        }
    }
}
