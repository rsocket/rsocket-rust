#[derive(Debug, Clone)]
pub enum URI {
    Tcp(String),
    Websocket(String),
}
