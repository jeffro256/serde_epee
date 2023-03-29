use std::fmt;

use serde::{de, ser};

pub type Result<T> = std::result::Result<T, Error>;

#[derive(Clone, Debug, PartialEq)]
pub enum ErrorKind {
	IOError,
	Custom,
	MissingFormatVersion,
	EnumVariantIndexTooBig,
	VarIntTooBig,
	VarIntTooSmall,
	SerdeModelUnsupported,
	TooManySectionFields,
	NoLength,
	KeyBadType,
	KeyBadEncoding,
	KeyTooLong,
	StringTooLong,
	StringBadEncoding,
	ArrayMixedTypes,
	NestedArrays,
	ArrayTooLong,
	TupleTooLong,
	BadTypeCode,
	ExpectedArray,
	ExpectedArrayEnd,
	ExpectedFormatSignature,
	ExpectedEnd,
	ExpectedScalar,
	NotExpectingArray,
	NotExpectingSection,
	NotExpectingScalar,
	BadUnicodeScalar,
	SizeHintMismatch,
	CompoundMissingArrayType,
	EmptySectionKey,
	TypeMismatch,
}

#[derive(Debug)]
pub struct Error {
	kind: ErrorKind,
	msg: String,
	source: Option<Box<dyn std::error::Error>>
}

///////////////////////////////////////////////////////////////////////////////

impl Error {
	pub fn new(kind: ErrorKind, msg: String) -> Self {
		Self { kind: kind, msg: msg, source: None }
	}

	pub fn new_no_msg(kind: ErrorKind) -> Self {
		Self { kind: kind, msg: String::from(""), source: None }
	}

	pub fn kind(&self) -> ErrorKind {
		self.kind.clone()
	}
}

///////////////////////////////////////////////////////////////////////////////
// Required traits for serde Serializer/Deserializer                         //
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

impl std::error::Error for Error {
	fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
		match &self.source {
			Some(s) => Some(s.as_ref()),
			None => None
		}
	}
}

///////////////////////////////////////////////////////////////////////////////
// Try/From trait implementations for convenience                            //
///////////////////////////////////////////////////////////////////////////////

impl From<std::io::Error> for Error {
	fn from(ioe: std::io::Error) -> Self {
		Self {
			kind: ErrorKind::IOError,
			msg: ioe.to_string(),
			source: Some(Box::new(ioe))
		}
	}
}

// Convenience macro
#[macro_export]
macro_rules! epee_err {
	($kind:ident) => (
		Err(Error::new_no_msg(ErrorKind::$kind))
	);
	($kind:ident, $fmt:expr, $($fmt_args:expr), *) => (
		Err(Error::new(ErrorKind::$kind, format!($fmt, $($fmt_args), *)))
	);
	($kind:ident, $msg:expr) => (
		Err(Error::new(ErrorKind::$kind, $msg.to_string()))
	)
}

pub(crate) use epee_err;
