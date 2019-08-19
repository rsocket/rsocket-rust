use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ErrorKind {
  WithDescription(&'static str),
  IO(io::Error),
}

#[derive(Debug)]
pub struct RSocketError {
  kind: ErrorKind,
}

impl Error for RSocketError {
  fn description(&self) -> &str {
    unimplemented!()
  }

  fn cause(&self) -> Option<&dyn Error> {
    unimplemented!()
  }
}

impl fmt::Display for RSocketError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    unimplemented!()
  }
}

impl From<io::Error> for RSocketError {
  fn from(e: io::Error) -> RSocketError {
    RSocketError {
      kind: ErrorKind::IO(e),
    }
  }
}
