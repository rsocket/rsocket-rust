use std::result;
use crate::errors::RSocketError;

pub type Result<T> = result::Result<T, RSocketError>;
