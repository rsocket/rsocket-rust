use std::error::Error;
use std::fmt;
use std::io;

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

#[derive(Debug)]
pub enum ErrorKind {
    Internal(u32, String),
    WithDescription(String),
    IO(io::Error),
    Cancelled(),
}

#[derive(Debug)]
pub struct RSocketError {
    kind: ErrorKind,
}

impl Error for RSocketError {}

impl fmt::Display for RSocketError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match &self.kind {
            ErrorKind::Internal(c, s) => write!(f, "ERROR({}): {}", translate(c), s),
            ErrorKind::WithDescription(s) => write!(f, "{}", s),
            ErrorKind::IO(e) => write!(f, "{}", e),
            ErrorKind::Cancelled() => write!(f, "ERROR(CANCELLED)"),
        }
    }
}

impl From<ErrorKind> for RSocketError {
    fn from(kind: ErrorKind) -> RSocketError {
        RSocketError { kind }
    }
}
impl From<String> for RSocketError {
    fn from(e: String) -> RSocketError {
        RSocketError {
            kind: ErrorKind::WithDescription(e),
        }
    }
}

impl From<&'static str> for RSocketError {
    fn from(e: &'static str) -> RSocketError {
        RSocketError {
            kind: ErrorKind::WithDescription(String::from(e)),
        }
    }
}

#[inline]
fn translate(code: &u32) -> &str {
    match *code {
        ERR_APPLICATION => "APPLICATION",
        ERR_INVALID_SETUP => "INVALID_SETUP",
        ERR_UNSUPPORTED_SETUP => "UNSUPPORTED_SETUP",
        ERR_REJECT_SETUP => "REJECT_SETUP",
        ERR_REJECT_RESUME => "REJECT_RESUME",
        ERR_CONN_FAILED => "CONN_FAILED",
        ERR_CONN_CLOSED => "CONN_CLOSED",
        ERR_REJECTED => "REJECTED",
        ERR_CANCELED => "CANCELED",
        ERR_INVALID => "INVALID",
        _ => "UNKNOWN",
    }
}
