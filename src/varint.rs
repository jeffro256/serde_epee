use std::convert::{Into, From, TryInto};
use std::io::{Read, Write};
use crate::error::{Error, ErrorKind, Result};

const MAX_BYTE_VAL:  u64 =                  63;
const MAX_WORD_VAL:  u64 =               16383;
const MAX_DWORD_VAL: u64 =          1073741823;
const MAX_QWORD_VAL: u64 = 4611686018427387903; 

pub struct VarInt {
	value: u64,
}

impl VarInt {
	pub fn to_writer<W: Write>(&self, writer: &mut W) -> Result<()> {
		Ok(())
	}

	pub fn from_reader<R: Read>(&self, reader: &mut R) -> Self {
		Self { value: 0 }
	}

	pub fn fits_byte(&self) -> bool {
		self.value <= MAX_BYTE_VAL
	}

	pub fn fits_word(&self) -> bool {
		self.value <= MAX_WORD_VAL
	}

	pub fn fits_dword(&self) -> bool {
		self.value <= MAX_DWORD_VAL
	}

	pub fn fits_qword(&self) -> bool {
		self.value <= MAX_QWORD_VAL
	}
}

impl TryInto<u8> for VarInt {
	type Error = Error;

	fn try_into(self) -> Result<u8> {
		if self.value <= MAX_BYTE_VAL {
			Ok(self.value as u8)
		} else {
			Err(Error::new_no_msg(ErrorKind::VarIntTooBig))
		}
	}
}

impl Into<u64> for VarInt {
	fn into(self) -> u64 {
		self.value
	}
}

impl From<u64> for VarInt {
	fn from(value: u64) -> Self {
		Self { value: value }
	}
}