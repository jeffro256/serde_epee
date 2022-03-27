use std::fmt;

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
	IOError(std::io::ErrorKind),
	Custom,
	MissingFormatVersion,
	EnumVariantIndexTooBig,
	VarIntTooBig,
	SerdeModelUnsupported
}

#[derive(Clone, Debug, PartialEq)]
pub struct Error {
	kind: ErrorKind,
	msg: String
}

///////////////////////////////////////////////////////////////////////////////

impl Error {
	pub fn new(kind: ErrorKind, msg: String) -> Self {
		Self { kind: kind, msg: msg }
	}

	pub fn new_no_msg(kind: ErrorKind) -> Self {
		Self { kind: kind, msg: String::from("") }
	}
}

///////////////////////////////////////////////////////////////////////////////
// Required traits for serde                                                 //
///////////////////////////////////////////////////////////////////////////////

impl ser::Error for Error {
	fn custom<T: fmt::Display>(msg: T) -> Self {
		Error::new(ErrorKind::Custom, msg.to_string())
	}
}

impl de::Error for Error {
	fn custom<T: fmt::Display>(msg: T) -> Self {
		Error::new(ErrorKind::Custom, msg.to_string())
	}
}

impl fmt::Display for Error {
	fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
		formatter.write_fmt(format_args!("{:?}: {}", self.kind, self.msg))
	}
}

impl std::error::Error for Error {}

///////////////////////////////////////////////////////////////////////////////
// Try/From trait implementations for convenience                            //
///////////////////////////////////////////////////////////////////////////////

impl From<std::io::Error> for Error {
	fn from(ioe: std::io::Error) -> Self {
		Self { kind: ErrorKind::IOError(ioe), msg: String::from("IOError") }
	}
}

/*
impl From<std::num::TryFromIntError> for Error {
	fn from(tre: std::num::TryFromIntError) -> Self {
		Self { kind: ErrorKind::TryFromIntError, msg: String::from("TryFromIntError (from u64)") }
	}
}
*/