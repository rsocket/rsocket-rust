use std::result;
use crate::errors::RSocketError;

pub type RSocketResult<T> = result::Result<T, RSocketError>;
