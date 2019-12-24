mod codec;
mod misc;
mod socket;
mod spi;
pub(crate) mod tcp;
pub(crate) use codec::RFrameCodec;
pub(crate) use socket::DuplexSocket;
pub(crate) use spi::*;
