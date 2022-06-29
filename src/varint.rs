use std::convert::{From, Into, TryFrom, TryInto};

use crate::error::{Error, ErrorKind, Result};

const MAX_BYTE_VAL:   u64 =                  63;
const MAX_WORD_VAL:   u64 =               16383;
const MAX_DWORD_VAL:  u64 =          1073741823;
const MAX_QWORD_VAL:  u64 = 4611686018427387903; 
const MAX_VARINT_VAL: u64 = MAX_QWORD_VAL;

#[derive(Debug)]
pub struct VarInt {
	value: u64,
}

impl VarInt {
	///////////////////////////////////////////////////////////////////////////////
	// Raw Read/Write methods                                                    //
	///////////////////////////////////////////////////////////////////////////////

	pub fn to_writer<W: std::io::Write>(&self, writer: &mut W) -> Result<()> {
		let (var_mask, byte_size) = if self.value <= MAX_BYTE_VAL {
			(0b00, 1)
		} else if self.value <= MAX_WORD_VAL {
			(0b01, 2)
		} else if self.value <= MAX_DWORD_VAL {
			(0b10, 4)
		} else {
			(0b11, 8)
		};

		let encoded = ((self.value << 2) | var_mask).to_le_bytes();

		let write_res = writer.write_all(&encoded[..byte_size]);
		match write_res {
			Ok(_) => Ok(()),
			Err(ioe) => Err(ioe.into())
		}
	}

	pub fn from_reader<R: std::io::Read>(reader: &mut R) -> Result<Self> {
		let mut buf = [0u8; 8];
		if let Err(ioe) = reader.read_exact(&mut buf[..1]) {
			return Err(ioe.into());
		}

		let var_mask = buf[0] & 0b11;
		let byte_size = 1 << var_mask;

		if let Err(ioe) = reader.read_exact(&mut buf[1..byte_size]) {
			return Err(ioe.into());
		}

		Ok(Self { value: u64::from_le_bytes(buf) >> 2 })
	}
}

///////////////////////////////////////////////////////////////////////////////
// Integer conversions                                                       //
///////////////////////////////////////////////////////////////////////////////

impl TryInto<u8> for VarInt {
	type Error = Error;

	fn try_into(self) -> Result<u8> {
		if self.value <= u8::MAX as u64 {
			Ok(self.value as u8)
		} else {
			Err(Error::new_no_msg(ErrorKind::VarIntTooBig))
		}
	}
}

impl TryInto<u16> for VarInt {
	type Error = Error;

	fn try_into(self) -> Result<u16> {
		if self.value <= u16::MAX as u64 {
			Ok(self.value as u16)
		} else {
			Err(Error::new_no_msg(ErrorKind::VarIntTooBig))
		}
	}
}

impl TryInto<u32> for VarInt {
	type Error = Error;

	fn try_into(self) -> Result<u32> {
		if self.value <= u32::MAX as u64 {
			Ok(self.value as u32)
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

impl TryInto<usize> for VarInt {
	type Error = Error;

	fn try_into(self) -> Result<usize> {
		if self.value <= usize::MAX as u64  {
			Ok(self.value as usize)
		} else {
			Err(Error::new_no_msg(ErrorKind::VarIntTooBig))
		}
	}
}

impl From<u8> for VarInt {
	fn from(value: u8) -> Self {
		Self { value: value.into() }
	}
}

impl From<u16> for VarInt {
	fn from(value: u16) -> Self {
		Self { value: value.into() }
	}
}

impl From<u32> for VarInt {
	fn from(value: u32) -> Self {
		Self { value: value.into() }
	}
}

impl TryFrom<u64> for VarInt {
	type Error = Error;

	fn try_from(value: u64) -> Result<Self> {
		if value <= MAX_VARINT_VAL {
			Ok(Self { value: value })
		} else {
			Err(Error::new(ErrorKind::VarIntTooSmall, String::from("u64 value exceeds maximum varint value")))
		}
	}
}

impl TryFrom<usize> for VarInt {
	type Error = Error;

	fn try_from(value: usize) -> Result<Self> {
		if (value as u64) <= MAX_VARINT_VAL {
			Ok(Self { value: value as u64 })
		} else {
			Err(Error::new(ErrorKind::VarIntTooSmall, String::from("usize value exceeds maximum varint value")))
		}
	}
}