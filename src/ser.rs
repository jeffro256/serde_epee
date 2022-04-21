use std::io::Write;
use serde::{ser, Serialize};

use crate::error::{Error, ErrorKind, Result};
use crate::constants;
use crate::varint::VarInt;

///////////////////////////////////////////////////////////////////////////////
// User functions                                                            //
///////////////////////////////////////////////////////////////////////////////

pub fn serialize_into<T, W>(value: &T, writer: &mut W) -> Result<()>
where
	T: Serialize,
	W: Write
{
	let mut serializer = Serializer::new_unstarted(writer)?;
	value.serialize(&mut serializer)
}

pub fn to_bytes<T: Serialize>(value: &T) -> Result<Vec<u8>> {
	let mut byte_stream = Vec::<u8>::new(); // Vec<u8> implements Write
	let mut serializer = Serializer::new_unstarted(&mut byte_stream)?;
	value.serialize(&mut serializer)?;
	Ok(byte_stream)
}

///////////////////////////////////////////////////////////////////////////////
// Serializer                                                                //
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, PartialEq)]
enum EpeeStorageFormat {
	Section,
	RootSection,
	Array,
	Packed,
	Unstarted
}

#[derive(Debug)]
pub struct Serializer<'a, W: Write> {
	writer: &'a mut W,
	storage_format: EpeeStorageFormat,
	len: u32,
	element_type: u8, // only important for arrays to enforce type consistency
	started: bool,
	serializing_key: bool
}

impl<'a, W> Serializer<'a, W>
where
	W: Write
{
	///////////////////////////////////////////////////////////////////////////////
	// Constructors                                                              //
	///////////////////////////////////////////////////////////////////////////////

	pub fn new_section(writer: &'a mut W, len: u32) -> Result<Self> {
		if len <= constants::MAX_NUM_SECTION_FIELDS as u32 {
			Ok(Self {
				writer: writer, 
				storage_format: EpeeStorageFormat::Section,
				len: len,
				element_type: constants::SERIALIZE_TYPE_UNKNOWN,
				started: false,
				serializing_key: false
			})
		} else {
			Err(Error::new(ErrorKind::TooManySectionFields, String::from("trying to serialize section with too many fields")))
		}
	}

	pub fn new_root_section(writer: &'a mut W, len: u32) -> Result<Self> {
		if len <= constants::MAX_NUM_SECTION_FIELDS as u32 {
			Ok(Self {
				writer: writer, 
				storage_format: EpeeStorageFormat::RootSection,
				len: len,
				element_type: constants::SERIALIZE_TYPE_UNKNOWN,
				started: false,
				serializing_key: false
			})
		} else {
			Err(Error::new(ErrorKind::TooManySectionFields, String::from("trying to serialize section with too many fields")))
		}
	}

	pub fn new_array(writer: &'a mut W, len: u32) -> Result<Self> {
		if len <= constants::MAX_NUM_SECTION_FIELDS as u32 {
			Ok(Self {
				writer: writer, 
				storage_format: EpeeStorageFormat::Array,
				len: len,
				element_type: constants::SERIALIZE_TYPE_UNKNOWN,
				started: false,
				serializing_key: false
			})
		} else {
			Err(Error::new(ErrorKind::TooManySectionFields, String::from("trying to serialize section with too many fields")))
		}
	}

	pub fn new_packed(writer: &'a mut W, len: u32) -> Result<Self> {
		if len <= constants::MAX_NUM_SECTION_FIELDS as u32 {
			Ok(Self {
				writer: writer, 
				storage_format: EpeeStorageFormat::Packed,
				len: len,
				element_type: constants::SERIALIZE_TYPE_UNKNOWN,
				started: false,
				serializing_key: false
			})
		} else {
			Err(Error::new(ErrorKind::TooManySectionFields, String::from("trying to serialize section with too many fields")))
		}
	}

	fn new_unstarted(writer: &'a mut W) -> Result<Self> {
		Ok(Self {
			writer: writer, 
			storage_format: EpeeStorageFormat::Unstarted,
			len: 0,
			element_type: constants::SERIALIZE_TYPE_UNKNOWN,
			started: false,
			serializing_key: false
		})
	}

	///////////////////////////////////////////////////////////////////////////////
	// Other methods                                                             //
	///////////////////////////////////////////////////////////////////////////////

	fn write_raw(&mut self, bytes: &[u8]) -> Result<()> {
		let write_res = self.writer.write_all(bytes);
		match write_res {
			Ok(_) => Ok(()),
			Err(ioe) => Err(ioe.into())
		}
	}

	fn write_type_code(&mut self, type_code: u8, is_array: bool) -> Result<()> {
		let array_mask = if is_array { constants::SERIALIZE_FLAG_ARRAY } else { 0 }; 
		let type_byte = [type_code | array_mask];
		self.write_raw(&type_byte).into()
	}

	// Format: one unsigned byte for the length, then the rest of the string, max 255 bytes
	fn write_key_string(&mut self, s: &[u8]) -> Result<()> {
		if s.len() > constants::MAX_SECTION_KEY_SIZE {
			return Err(Error::new_no_msg(ErrorKind::KeyTooLong));
		}

		let len = s.len() as u8;
		self.write_raw(&[len])?;
		self.write_raw(s)
	}

	fn serialize_start_and_type_code(&mut self, type_code: u8) -> Result<()> {
		if !self.started {
			match &self.storage_format {
				EpeeStorageFormat::Section => self.write_type_code(constants::SERIALIZE_TYPE_OBJECT, false)?,
				EpeeStorageFormat::RootSection => self.write_raw(&constants::PORTABLE_STORAGE_SIGNATURE)?,
				EpeeStorageFormat::Array => self.write_type_code(type_code, true)?,
				EpeeStorageFormat::Packed => (),
				EpeeStorageFormat::Unstarted => (),
			};

			if self.storage_format != EpeeStorageFormat::Packed {
				let varlen = VarInt::from(self.len);
				varlen.to_writer(self.writer)?;
			}

			self.element_type = type_code;
			self.started = true;
		}

		if self.storage_format == EpeeStorageFormat::Array && type_code != self.element_type {
			let msg = format!("type_codes: {} -> {}", self.element_type, type_code);
			return Err(Error::new(ErrorKind::ArrayMixedTypes, msg));
		} else if self.serializing_key && type_code != constants::SERIALIZE_TYPE_STRING {
			return Err(Error::new_no_msg(ErrorKind::KeyBadType))
		}

		if (self.storage_format == EpeeStorageFormat::Section || self.storage_format == EpeeStorageFormat::RootSection)
				&& type_code != constants::SERIALIZE_TYPE_UNKNOWN
		{
			self.write_type_code(type_code, false)?;
		}

		Ok(())
	}
}

macro_rules! serialize_num {
	($fname:ident, $numtype:ty, $numcode:expr) => (
		fn $fname(self, v: $numtype) -> Result<()> {
			self.serialize_start_and_type_code($numcode)?;
			self.write_raw(&v.to_le_bytes())
		}
	)
}

impl<'b, 'a: 'b, W> ser::Serializer for &'b mut Serializer<'a, W>
where
	W: Write
{
	type Ok = ();
	type Error = Error;

	type SerializeSeq = Serializer<'b, W>;
	type SerializeTuple = Serializer<'b, W>;
	type SerializeTupleStruct = Serializer<'b, W>;
	type SerializeTupleVariant = Serializer<'b, W>;
	type SerializeMap = Serializer<'b, W>;
	type SerializeStruct = Serializer<'b, W>;
	type SerializeStructVariant = Serializer<'b, W>;

	serialize_num!{serialize_i8, i8, constants::SERIALIZE_TYPE_INT8}
	serialize_num!{serialize_i16, i16, constants::SERIALIZE_TYPE_INT16}
	serialize_num!{serialize_i32, i32, constants::SERIALIZE_TYPE_INT32}
	serialize_num!{serialize_i64, i64, constants::SERIALIZE_TYPE_INT64}
	serialize_num!{serialize_u8, u8, constants::SERIALIZE_TYPE_UINT8}
	serialize_num!{serialize_u16, u16, constants::SERIALIZE_TYPE_UINT16}
	serialize_num!{serialize_u32, u32, constants::SERIALIZE_TYPE_UINT32}
	serialize_num!{serialize_u64, u64, constants::SERIALIZE_TYPE_UINT64}
	serialize_num!{serialize_f64, f64, constants::SERIALIZE_TYPE_DOUBLE}

	fn serialize_bool(self, v: bool) -> Result<()> {
		self.serialize_start_and_type_code(constants::SERIALIZE_TYPE_BOOL)?;
		self.write_raw(&[v as u8])
	}

	fn serialize_f32(self, v: f32) -> Result<()> {
		self.serialize_f64(v as f64)
	}

	fn serialize_char(self, v: char) -> Result<()> {
		self.serialize_u32(v as u32)
	}

	fn serialize_str(self, v: &str) -> Result<()> {
		self.serialize_bytes(v.as_bytes())
	}

	// EPEE "Blob"
	fn serialize_bytes(self, v: &[u8]) -> Result<()> {
		if self.serializing_key {
			let res = self.write_key_string(v);
			self.serializing_key = false;
			return res;
		} else {
			if v.len() > constants::MAX_STRING_LEN_POSSIBLE {
				return Err(Error::new_no_msg(ErrorKind::StringTooLong));
			}

			self.serialize_start_and_type_code(constants::SERIALIZE_TYPE_STRING)?;

			let varlen = VarInt::try_from(v.len()).unwrap();
			varlen.to_writer(self.writer)?;

			return self.write_raw(v);
		}
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
			_variant_index: u32,
			_variant: &'static str
	) -> Result<()> {
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("can't serialize unit variants")))
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
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
		_value: &T,
	) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("can't serialize unit variants")))
	}

	///////////////////////////////////////////////////////////////////////////
	// Delegate Compound Types                                               //
	///////////////////////////////////////////////////////////////////////////

	fn serialize_seq(self, len: Option<usize>) -> Result<Self::SerializeSeq> {
		if self.storage_format == EpeeStorageFormat::Array {
			return Err(Error::new_no_msg(ErrorKind::NestedArrays));
		}

		if let Some(l) = len {
			if l <= constants::MAX_NUM_SECTION_FIELDS {
				Serializer::new_array(self.writer, l as u32)
			} else {
				Err(Error::new_no_msg(ErrorKind::ArrayTooLong))
			}
		} else  {
			Err(Error::new(ErrorKind::NoLength, String::from("EPEE serializer needs to know seq length ahead of time")))
		}
	}

	fn serialize_tuple(self, len: usize) -> Result<Self::SerializeTuple> {
		if len <= constants::MAX_NUM_SECTION_FIELDS {
			Serializer::new_packed(self.writer, len as u32)
		} else {
			Err(Error::new_no_msg(ErrorKind::TupleTooLong))
		}
	}

	fn serialize_tuple_struct(
		self,
		_name: &'static str,
		len: usize,
	) -> Result<Self::SerializeTupleStruct> {
		self.serialize_tuple(len)
	}

	fn serialize_tuple_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeTupleVariant> {
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("can't serialize tuple variants")))
	}

	fn serialize_map(self, len: Option<usize>) -> Result<Self::SerializeMap> {
		match len {
			Some(l) => {
				match &self.storage_format {
					EpeeStorageFormat::Unstarted => Serializer::new_root_section(self.writer, l as u32),
					_ => Serializer::new_section(self.writer, l as u32)
				}
			},
			None => Err(Error::new(ErrorKind::NoLength, String::from("EPEE serializer needs to know map length ahead of time")))
		}
	}

	fn serialize_struct(
		self,
		_name: &'static str,
		len: usize,
	) -> Result<Self::SerializeStruct> {
		self.serialize_map(Some(len))
	}

	// Struct variants are represented in JSON as `{ NAME: { K: V, ... } }`.
	// This is the externally tagged representation.
	fn serialize_struct_variant(
		self,
		_name: &'static str,
		_variant_index: u32,
		_variant: &'static str,
		_len: usize,
	) -> Result<Self::SerializeStructVariant> {
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("can't serialize struct variants")))
	}
}

///////////////////////////////////////////////////////////////////////////
// Implementations for supported compound types                          //
///////////////////////////////////////////////////////////////////////////

impl<'a, W> ser::SerializeSeq for Serializer<'a, W>
where 
	W: Write
{
	type Ok = ();
	type Error = Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<()>
	where
		T: ?Sized + ser::Serialize,
	{
		value.serialize(self)
	}

	// @TODO: enforce length of serialized compound
	fn end(self) -> Result<()> {
		Ok(())
	}
}

// Same as SerializeSeq
impl<'a, W> ser::SerializeTuple for Serializer<'a, W>
where
	W: Write	
{
	type Ok = ();
	type Error = Error;

	fn serialize_element<T>(&mut self, value: &T) -> Result<()>
	where
		T: ?Sized + ser::Serialize,
	{
		value.serialize(self)
	}

	// @TODO: enforce length of serialized compound
	fn end(self) -> Result<()> {
		Ok(())
	}
}

// Same as SerializeSeq
impl<'a, W> ser::SerializeTupleStruct for Serializer<'a, W>
where
	W: Write
{
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, value: &T) -> Result<()>
	where
		T: ?Sized + ser::Serialize,
	{
		value.serialize(self)
	}

	// @TODO: enforce length of serialized compound
	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a, W> ser::SerializeMap for Serializer<'a, W>
where
	W: Write
{
	type Ok = ();
	type Error = Error;

	fn serialize_key<T>(&mut self, key: &T) -> Result<()>
	where
		T: ?Sized + ser::Serialize,
	{
		self.serialize_start_and_type_code(constants::SERIALIZE_TYPE_UNKNOWN)?;

		// Man I really need specialization
		self.serializing_key = true;
		key.serialize(self)?;
		// serializing_key set to false it serialize_bytes()

		Ok(())
	}

	fn serialize_value<T>(&mut self, value: &T) -> Result<()>
	where
		T: ?Sized + ser::Serialize,
	{
		value.serialize(self)
	}

	// @TODO: enforce length of serialized compound
	fn end(self) -> Result<()> {
		Ok(())
	}
}

impl<'a, W> ser::SerializeStruct for Serializer<'a, W>
where
	W: Write
{
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, key: &'static str, value: &T) -> Result<()>
	where
		T: ?Sized + ser::Serialize,
	{
		self.serialize_start_and_type_code(constants::SERIALIZE_TYPE_UNKNOWN)?;

		self.write_key_string(key.as_bytes())?;
		value.serialize(self)
	}

	// @TODO: enforce length of serialized compound
	fn end(self) -> Result<()> {
		Ok(())
	}
}

///////////////////////////////////////////////////////////////////////////
// Empty implementations for unsupported compound types                  //
///////////////////////////////////////////////////////////////////////////

impl<'a, W> ser::SerializeTupleVariant for Serializer<'a, W>
where
	W: Write
{
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, _value: &T) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("can't serialize tuple variants")))
	}

	fn end(self) -> Result<()> {
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("can't serialize tuple variants")))
	}
}

impl<'a, W> ser::SerializeStructVariant for Serializer<'a, W>
where
	W: Write
{
	type Ok = ();
	type Error = Error;

	fn serialize_field<T>(&mut self, _key: &'static str, _value: &T) -> Result<()>
	where
		T: ?Sized + Serialize,
	{
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("can't serialize struct variants")))
	}

	fn end(self) -> Result<()> {
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("can't serialize struct variants")))
	}
}