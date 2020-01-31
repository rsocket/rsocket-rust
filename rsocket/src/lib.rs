#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;
#[macro_use]
extern crate matches;

pub mod extension;

pub mod frame;
pub mod mime;

// mod core;
mod errors;
mod payload;
mod result;
mod spi;

pub mod transport;

mod x;

pub mod prelude {
    pub use crate::errors::*;
    pub use crate::payload::*;
    pub use crate::result::*;
    pub use crate::spi::*;
    pub use crate::transport::{ClientTransport, Rx, ServerTransport, Tx};
    pub use crate::x::{Client, RSocketFactory};
    pub use futures::{Sink, SinkExt, Stream, StreamExt};
}
