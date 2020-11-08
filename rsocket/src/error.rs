use std::fmt;
use std::io;
use thiserror::Error;

pub const ERR_INVALID_SETUP: u32 = 0x0000_0001;
pub const ERR_UNSUPPORTED_SETUP: u32 = 0x0000_0002;
pub const ERR_REJECT_SETUP: u32 = 0x0000_0003;
pub const ERR_REJECT_RESUME: u32 = 0x0000_0004;
pub const ERR_CONN_FAILED: u32 = 0x0000_0101;
pub const ERR_CONN_CLOSED: u32 = 0x0000_0102;
pub const ERR_APPLICATION: u32 = 0x0000_0201;
pub const ERR_REJECTED: u32 = 0x0000_0202;
pub const ERR_CANCELED: u32 = 0x0000_0203;
pub const ERR_INVALID: u32 = 0x0000_0204;

#[derive(Error, Debug)]
pub enum RSocketError {
    // Protocol errors:
    #[error("INVALID_SETUP: {0}")]
    InvalidSetup(String),
    #[error("UNSUPPORTED_SETUP: {0}")]
    UnsupportedSetup(String),
    #[error("REJECTED_SETUP: {0}")]
    RejectedSetup(String),
    #[error("REJECTED_SETUP: {0}")]
    RejectedResume(String),
    #[error("CONNECTION_ERROR: {0}")]
    ConnectionException(String),
    #[error("CONNECTION_CLOSE: {0}")]
    ConnectionClosed(String),
    #[error("APPLICATION_ERROR: {0}")]
    ApplicationException(String),
    #[error("REJECTED: {0}")]
    RequestRejected(String),
    #[error("CANCELLED: {0}")]
    RequestCancelled(String),
    #[error("INVALID: {0}")]
    RequestInvalid(String),
    #[error("RESERVED({0}): {1}")]
    Reserved(u32, String),

    // Codec errors:
    #[error("this frame is incomplete")]
    InCompleteFrame,
    // Custom errors:
    #[error("{0}")]
    WithDescription(String),
    #[error(transparent)]
    IO(#[from] io::Error),
    #[error(transparent)]
    Other(#[from] anyhow::Error),
}

impl RSocketError {
    pub(crate) fn must_new_from_code(code: u32, desc: String) -> Self {
        match code {
            ERR_APPLICATION => RSocketError::ApplicationException(desc),
            ERR_INVALID_SETUP => RSocketError::InvalidSetup(desc),
            ERR_UNSUPPORTED_SETUP => RSocketError::UnsupportedSetup(desc),
            ERR_REJECT_SETUP => RSocketError::RejectedSetup(desc),
            ERR_REJECT_RESUME => RSocketError::RejectedResume(desc),
            ERR_CONN_FAILED => RSocketError::ConnectionException(desc),
            ERR_CONN_CLOSED => RSocketError::ConnectionClosed(desc),
            ERR_REJECTED => RSocketError::RequestRejected(desc),
            ERR_CANCELED => RSocketError::RequestCancelled(desc),
            ERR_INVALID => RSocketError::RequestInvalid(desc),
            _ => RSocketError::Reserved(code, desc),
        }
    }
}
