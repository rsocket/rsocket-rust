use crate::errors::RSocketError;
use std::error::Error;
use std::future::Future;
use std::pin::Pin;
use std::result::Result;

pub const DEFAULT_MIME_TYPE: &str = "application/binary";

pub type RSocketResult<T> = Result<T, RSocketError>;
