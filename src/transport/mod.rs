mod codec;
mod conn;
mod misc;
mod spi;
pub(crate) mod tcp;
pub(crate) use codec::RFrameCodec;
pub use conn::DuplexSocket;
pub use spi::{Rx, Transport, Tx};
