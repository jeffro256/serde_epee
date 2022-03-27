use std::io::Write;
use serde::{ser, Serialize};
use crate::error::{Error, ErrorKind, Result};

#[derive(Debug)]
pub struct Serializer<W: Write + std::fmt::Debug> {
	writer: W
}

impl<W: Write + std::fmt::Debug> Serializer<W> {
	pub fn new(writer: W) -> Self {
		Self { writer: writer }
	}
}

pub fn to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
	let mut serializer = Serializer {
		writer: Vec::<u8>::new()        // Vec<u8> implements Write
	};
	value.serialize(&mut serializer)?;
	Ok(serializer.writer)
}

impl<'a, W: Write + std::fmt::Debug> ser::Serializer for &'a mut Serializer<W> {
	type Ok = ();
	type Error = Error;

	type SerializeSeq = Self;
	type SerializeTuple = Self;
	type SerializeTupleStruct = Self;
	type SerializeTupleVariant = Self;
	type SerializeMap = Self;
	type SerializeStruct = Self;
	type SerializeStructVariant = Self;

	fn serialize_bool(self, v: bool) -> Result<()> {
		let bool_byte = [u8::from(v)];
		self.writer.write_all(&bool_byte)
	}

	fn serialize_i8(self, v: i8) -> Result<()> {
		let int_bytes = v.to_le_bytes();
		self.writer.write_all(&int_bytes)
	}

	fn serialize_i16(self, v: i16) -> Result<()> {
		let int_bytes = v.to_le_bytes();
		self.writer.write_all(&int_bytes)
	}

	fn serialize_i32(self, v: i32) -> Result<()> {
		let int_bytes = v.to_le_bytes();
		self.writer.write_all(&int_bytes)
	}

	fn serialize_i64(self, v: i64) -> Result<()> {
		let int_bytes = v.to_le_bytes();
		self.writer.write_all(&int_bytes)
	}

	fn serialize_u8(self, v: u8) -> Result<()> {
		let int_bytes = v.to_le_bytes();
		self.writer.write_all(&int_bytes)
	}

	fn serialize_u16(self, v: u16) -> Result<()> {
		let int_bytes = v.to_le_bytes();
		self.writer.write_all(&int_bytes)
	}

	fn serialize_u32(self, v: u32) -> Result<()> {
		let int_bytes = v.to_le_bytes();
		self.writer.write_all(&int_bytes)
	}

	fn serialize_u64(self, v: u64) -> Result<()> {
		let int_bytes = v.to_le_bytes();
		self.writer.write_all(&int_bytes)
	}

	fn serialize_f32(self, v: f32) -> Result<()> {
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("can't serialize floats")))
	}

	fn serialize_f64(self, v: f64) -> Result<()> {
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("can't serialize doubles")))
	}

	fn serialize_char(self, v: char) -> Result<()> {
		let mut buf = [0u8; 4]; // Should be big enough for all Unicode scalar values
		let s = v.encode_utf8(&mut buf);
		self.serialize_str(s)
	}

	fn serialize_str(self, v: &str) -> Result<()> {
		self.serialize_bytes(v.as_bytes())
	}

	fn serialize_bytes(self, v: &[u8]) -> Result<()> {
		self.writer.write_all(v);
    }

	fn serialize_none(self) -> Result<()> {
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("can't serialize none")))
	}

	// Drop the optional wrapper: serialize Some(v) as v
	fn serialize_some<T>(self, value: &T) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(self)
	}

	fn serialize_unit(self) -> Result<()> {
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("can't serialize anonymous unit")))
	}

	fn serialize_unit_struct(self, _name: &'static str) -> Result<()> {
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("can't serialize unit structs")))
	}

	fn serialize_unit_variant(
			self,
			_name: &'static str,
			variant_index: u32,
			_variant: &'static str
	) -> Result<()> {
		if variant_index <= 255 {
			self.serialize_u8(variant_index as u8)
		} else {
			Err(Error::new(
				ErrorKind::EnumVariantIndexTooBig,
				String::from("EPEE serialization only supports enums of size 256 or less")
			))
		}
	}

	fn serialize_newtype_struct<T>(
		self,
		_name: &'static str,
		value: &T,
	) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		value.serialize(self)
	}

	fn serialize_newtype_variant<T>(
		self,
		name: &'static str,
		variant_index: u32,
		variant: &'static str,
		value: &T,
	) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		self.serialize_unit_variant(name, variant_index, variant)?
		value.serialize(&mut *self)?
	}
}