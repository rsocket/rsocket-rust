#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

mod errors;
pub mod extension;
pub mod frame;
pub mod mime;
mod misc;
mod payload;
pub mod runtime;
mod spi;
pub mod transport;
mod x;

pub mod prelude {
    pub use crate::errors::*;
    pub use crate::misc::RSocketResult;
    pub use crate::payload::*;
    pub use crate::runtime::Spawner;
    pub use crate::spi::*;
    pub use crate::transport::{ClientTransport, Rx, ServerTransport, Tx};
    pub use crate::x::{Client, RSocketFactory};
    pub use futures::{Sink, SinkExt, Stream, StreamExt};
}
