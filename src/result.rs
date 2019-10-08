use crate::errors::RSocketError;
use std::result;

pub type RSocketResult<T> = result::Result<T, RSocketError>;
