extern crate futures;

use futures::sync::oneshot;
use futures::sync::mpsc;
use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ErrorKind {
  WithDescription(&'static str),
  IO(io::Error),
  Cancelled(),
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
    println!(">>>>>>>>>>> {:?}", self.kind);
    unimplemented!()
  }
}

impl From<&'static str> for RSocketError {
  fn from(e: &'static str) -> RSocketError {
    RSocketError {
      kind: ErrorKind::WithDescription(e),
    }
  }
}

impl From<oneshot::Canceled> for RSocketError {
  fn from(e: oneshot::Canceled) -> RSocketError {
    RSocketError {
      kind: ErrorKind::Cancelled(),
    }
  }
}

impl From<io::Error> for RSocketError {
  fn from(e: io::Error) -> RSocketError {
    RSocketError {
      kind: ErrorKind::IO(e),
    }
  }
}
