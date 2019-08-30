#[derive(Debug,Clone)]
pub enum URI{
  Tcp(&'static str),
  Websocket(&'static str),
}
