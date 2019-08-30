use crate::x::{Client,ClientBuilder};
use crate::x::{ServerBuilder};

pub struct RSocketFactory{
}

impl RSocketFactory{
  pub fn connect() -> ClientBuilder{
    Client::builder()
  }
  pub fn receive() -> ServerBuilder{
    ServerBuilder::new()
  }
}
