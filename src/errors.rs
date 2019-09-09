extern crate futures;

use futures::sync::mpsc;
use futures::sync::oneshot;
use std::error::Error;
use std::fmt;
use std::io;

#[derive(Debug)]
pub enum ErrorKind {
  Internal(u32, &'static str),
  WithDescription(&'static str),
  IO(io::Error),
  Cancelled(),
  Send(),
}

#[derive(Debug)]
pub struct RSocketError {
  kind: ErrorKind,
}

impl Error for RSocketError {
  fn description(&self) -> &str {
    "this is a rsocket error"
  }

  fn cause(&self) -> Option<&dyn Error> {
    match &self.kind {
      ErrorKind::IO(e) => Some(e),
      _ => None,
    }
  }
}

impl fmt::Display for RSocketError {
  fn fmt(&self, f: &mut fmt::Formatter<'_>) -> Result<(), fmt::Error> {
    println!(">>>>>>>>>>> {:?}", self.kind);
    unimplemented!()
  }
}

impl From<ErrorKind> for RSocketError {
  fn from(kind: ErrorKind) -> RSocketError {
    RSocketError { kind }
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
    RSocketError::from(ErrorKind::Cancelled())
  }
}

impl<T> From<mpsc::SendError<T>> for RSocketError {
  fn from(_e: mpsc::SendError<T>) -> RSocketError {
    RSocketError::from(ErrorKind::Send())
  }
}

impl From<io::Error> for RSocketError {
  fn from(e: io::Error) -> RSocketError {
    RSocketError::from(ErrorKind::IO(e))
  }
}
