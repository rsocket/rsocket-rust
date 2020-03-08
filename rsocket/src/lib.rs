#![allow(dead_code)]
#![allow(unused_imports)]
#![allow(unused_variables)]
#![allow(clippy::type_complexity)]

#[macro_use]
extern crate log;
#[macro_use]
extern crate lazy_static;

pub mod error;
pub mod extension;

#[cfg(feature = "frame")]
pub mod frame;
#[cfg(not(feature = "frame"))]
mod frame;

pub mod mime;
mod payload;
pub mod runtime;
mod spi;
pub mod transport;
pub mod utils;
mod x;

pub mod prelude {
    pub use crate::payload::{Payload, PayloadBuilder, SetupPayload, SetupPayloadBuilder};
    pub use crate::runtime::Spawner;
    pub use crate::spi::*;
    pub use crate::transport::{ClientTransport, Rx, ServerTransport, Tx};
    pub use crate::utils::RSocketResult;
    pub use crate::x::{Client, RSocketFactory};
    pub use futures::{Sink, SinkExt, Stream, StreamExt};
}
