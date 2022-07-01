// @TODO Non UTF-8 string support is sketchy

use std::io::Read;

use serde::Deserialize;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};

use crate::constants;
use crate::error::{Error, ErrorKind, Result, epee_err};
use crate::VarInt;

///////////////////////////////////////////////////////////////////////////////
// User functions  (use these if you're new here)                            //
///////////////////////////////////////////////////////////////////////////////

pub fn from_reader<T, R>(mut reader: R) -> Result<T>
where
	T: de::DeserializeOwned,
	R: Read
{
	let mut deserializer = Deserializer::from_reader(&mut reader);
	T::deserialize(&mut deserializer)
}

pub fn from_bytes<'a, T>(bytes: &'a mut &[u8]) -> Result<T>
where
	T: Deserialize<'a>,
{
	let mut deserializer = Deserializer::from_reader(bytes);
	T::deserialize(&mut deserializer)
}

///////////////////////////////////////////////////////////////////////////////
// EPEE Type definitions                                                     //
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug, Copy, Clone, PartialEq)]
enum EpeeScalarType {
	Int64,
	Int32,
	Int16,
	Int8,
	UInt64,
	UInt32,
	UInt16,
	UInt8,
	Double,
	Str,
	Bool,
	Object
}

impl EpeeScalarType {
	fn from_type_code(type_code: u8) -> Result<Self> {
		const TYPES: [EpeeScalarType; 12] = [
			EpeeScalarType::Int64,
			EpeeScalarType::Int32,
			EpeeScalarType::Int16,
			EpeeScalarType::Int8,
			EpeeScalarType::UInt64,
			EpeeScalarType::UInt32,
			EpeeScalarType::UInt16,
			EpeeScalarType::UInt8,
			EpeeScalarType::Double,
			EpeeScalarType::Str,
			EpeeScalarType::Bool,
			EpeeScalarType::Object
		];

		let scalar_type_code = type_code & !constants::SERIALIZE_FLAG_ARRAY;

		if scalar_type_code == 0 || scalar_type_code > 12 {
			return epee_err!(BadTypeCode, "Invalid value: {}", type_code);
		}

		Ok(TYPES[scalar_type_code as usize - 1])
	}
}

#[derive(Debug)]
struct EpeeEntryType {
	scalar_type: EpeeScalarType,
	is_array: bool
}

impl EpeeEntryType {
	fn from_type_code(type_code: u8) -> Result<Self> {
		let scalar = EpeeScalarType::from_type_code(type_code)?;
		let is_array = 0 != (type_code & constants::SERIALIZE_FLAG_ARRAY);

		Ok(Self {
			scalar_type: scalar,
			is_array: is_array
		})
	}
}

///////////////////////////////////////////////////////////////////////////////
// Deserializer definition                                                   //
///////////////////////////////////////////////////////////////////////////////

#[derive(Debug)]
enum DeserState {
	ExpectingSection(bool), // true if expecting root section, false otherwise 
	ExpectingKey,
	ExpectingEntry,
	ExpectingScalar(EpeeScalarType),
	Done
}

pub struct Deserializer<'de, R: Read> {
	reader: &'de mut R,
	state: DeserState,
}

// Defines a method which parses a certain primitive number type raw from stream
// All the primitive types have a nofail from_le_bytes method but there is no trait
macro_rules! define_parse_num {
	( $fname:ident, $numtype:ty ) => {
		fn $fname(&mut self) -> Result<$numtype>
		{
			const NBYTES: usize = std::mem::size_of::<$numtype>();
			let mut le_bytes = [0u8; NBYTES];
			self.read_raw(&mut le_bytes)?;
			let num = <$numtype>::from_le_bytes(le_bytes);
			Ok(num)
		}
	}
}

// Defines a method to implement serde::Deserializer. All defs are the same since
// we ignore type hints from deserialize instance since epee is self-describing
macro_rules! define_simple_deser {
	( $fname:ident ) => {
		fn $fname<V>(self, visitor: V) -> Result<V::Value>
		where
			V: Visitor<'de>
		{
			self.deserialize_any(visitor)
		}
	}
}

impl<'de, R: Read> Deserializer<'de, R> {
	///////////////////////////////////////////////////////////////////////////////
	// Constructors                                                              //
	///////////////////////////////////////////////////////////////////////////////
	pub fn from_reader(reader: &'de mut R) -> Self {
		Self {
			reader: reader,
			state: DeserState::ExpectingSection(true)
		}
	}

	///////////////////////////////////////////////////////////////////////////////
	// Reading helpers                                                           //
	///////////////////////////////////////////////////////////////////////////////

	fn read_raw(&mut self, buf: &mut [u8]) -> Result<()> {
		let read_res = self.reader.read_exact(buf);
		match read_res { 
			Ok(_) => Ok(()),
			Err(ioe) => Err(ioe.into())
			//Err(ioe) => panic!("Error reading {} bytes", buf.len())
		}
	}

	fn read_single(&mut self) -> Result<u8> {
		let mut single_byte = [0u8];
		match self.reader.read_exact(&mut single_byte) {
			Ok(_) => Ok(single_byte[0]),
			Err(ioe) => Err(ioe.into())
		}
	}

	fn deserialize_section_entry<V>(&mut self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		let entry_type = self.parse_type_code()?;

		if entry_type.is_array {
			visitor.visit_seq(EpeeCompound::new_array(self, None, entry_type.scalar_type))
		} else {
			self.state = DeserState::ExpectingScalar(entry_type.scalar_type);
			self.deserialize_scalar(visitor)
		}
	}

	fn deserialize_scalar<V>(&mut self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		if let DeserState::ExpectingScalar(scalar_type) = self.state {
			match scalar_type {
				EpeeScalarType::Int64  => visitor.visit_i64   (self.parse_i64()?),
				EpeeScalarType::Int32  => visitor.visit_i32   (self.parse_i32()?),
				EpeeScalarType::Int16  => visitor.visit_i16   (self.parse_i16()?),
				EpeeScalarType::Int8   => visitor.visit_i8    (self.parse_i8()?),
				EpeeScalarType::UInt64 => visitor.visit_u64   (self.parse_u64()?),
				EpeeScalarType::UInt32 => visitor.visit_u32   (self.parse_u32()?),
				EpeeScalarType::UInt16 => visitor.visit_u16   (self.parse_u16()?),
				EpeeScalarType::UInt8  => visitor.visit_u8    (self.parse_u8()?),
				EpeeScalarType::Double => visitor.visit_f64   (self.parse_f64()?),
				EpeeScalarType::Str    => visitor.visit_bytes (self.parse_string_value()?.as_slice()),
				EpeeScalarType::Bool   => visitor.visit_bool  (self.parse_bool()?),
				EpeeScalarType::Object => visitor.visit_map   (EpeeCompound::new_section(self, None))
			}
		} else {
			epee_err!(ExpectedScalar)
		}
	}

	///////////////////////////////////////////////////////////////////////////////
	// Parsing (note: number parsing is handled by deserialize_num macro)        //
	///////////////////////////////////////////////////////////////////////////////

	fn parse_type_code(&mut self) -> Result<EpeeEntryType> {
		EpeeEntryType::from_type_code(self.read_single()?)
	}

	fn parse_bool(&mut self) -> Result<bool> {
		let bool_byte = self.read_single()?;
		Ok(bool_byte != 0)
	}

	fn parse_char(&mut self) -> Result<char> {
		let mut scalar_buf = [0u8; 4];
		self.read_raw(&mut scalar_buf)?;
		let scalar_val = u32::from_le_bytes(scalar_buf);
		match scalar_val.try_into() {
			Ok(c) => Ok(c),
			Err(_) => epee_err!(BadUnicodeScalar, "Deserialized invalid unicode scalar value: {:#10x}", scalar_val)
		}
	}

	// @TODO construct string reference with class lifetime to avoid copying
	// for section keys
	fn parse_string_key(&mut self) -> Result<String> {
		let strlen = self.read_single()? as usize;
		if strlen == 0 {
			return epee_err!(EmptySectionKey, "section key length can not be zero!");
		}
		let mut strbuf = vec![0u8; strlen];
		self.read_raw(strbuf.as_mut_slice())?;
		match String::from_utf8(strbuf) {
			Ok(s) => Ok(s),
			Err(_) => epee_err!(StringBadEncoding, "UTF-8 encoding error while parsing byte buffer for string key")
		}
	}

	// @TODO construct string reference with class lifetime to avoid copying
	// for normal string values of type SERIALIZE_TYPE_STRING
	fn parse_string_value(&mut self) -> Result<Vec<u8>> {
		let varlen = VarInt::from_reader(self.reader)?;
		let strsize: usize = varlen.try_into()?;
		if strsize > constants::MAX_STRING_LEN_POSSIBLE {
			return Err(Error::new_no_msg(ErrorKind::StringTooLong))
		}

		// @TODO: We may not want to allocate the whole string in advance for resource security against bad connections
		let mut strbuf = vec![0u8; strsize];
		self.read_raw(strbuf.as_mut_slice())?;
		Ok(strbuf)
	}

	define_parse_num!{parse_u8, u8}
	define_parse_num!{parse_u16, u16}
	define_parse_num!{parse_u32, u32}
	define_parse_num!{parse_u64, u64}
	define_parse_num!{parse_i8, i8}
	define_parse_num!{parse_i16, i16}
	define_parse_num!{parse_i32, i32}
	define_parse_num!{parse_i64, i64}
	define_parse_num!{parse_f64, f64}
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut Deserializer<'de, R> {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		match self.state {
			DeserState::ExpectingSection(true) => visitor.visit_map(EpeeCompound::new_root_section(self, None)),
			DeserState::ExpectingSection(false) => visitor.visit_map(EpeeCompound::new_section(self, None)),
			DeserState::ExpectingKey => visitor.visit_str(self.parse_string_key()?.as_str()),
			DeserState::ExpectingEntry => self.deserialize_section_entry(visitor),
			DeserState::ExpectingScalar(_) => self.deserialize_scalar(visitor),
			DeserState::Done => epee_err!(ExpectedEnd, "deserialize_any() was called after Deserializer was done")
		}
	}

	define_simple_deser!{deserialize_bool}
	define_simple_deser!{deserialize_u8}
	define_simple_deser!{deserialize_u16}
	define_simple_deser!{deserialize_u32}
	define_simple_deser!{deserialize_u64}
	define_simple_deser!{deserialize_i8}
	define_simple_deser!{deserialize_i16}
	define_simple_deser!{deserialize_i32}
	define_simple_deser!{deserialize_i64}
	define_simple_deser!{deserialize_f32}
	define_simple_deser!{deserialize_f64}
	define_simple_deser!{deserialize_str}
	define_simple_deser!{deserialize_string}
	define_simple_deser!{deserialize_identifier}
	define_simple_deser!{deserialize_ignored_any}
	define_simple_deser!{deserialize_seq}
	define_simple_deser!{deserialize_map}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		unimplemented!()
	}

	// The `Serializer` implementation on the previous page serialized byte
	// arrays as JSON arrays of bytes. Handle that representation here.
	fn deserialize_bytes<V>(self, _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		unimplemented!()
	}

	fn deserialize_byte_buf<V>(self, _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		unimplemented!()
	}

	fn deserialize_option<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_some(self)
	}

	fn deserialize_unit<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_unit()
	}

	fn deserialize_unit_struct<V>(
		self,
		_name: &'static str,
		_visitor: V,
	) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("Can't deserialize unit structs")))
	}

	fn deserialize_newtype_struct<V>(
		self,
		_name: &'static str,
		_visitor: V,
	) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("Can't deserialize newtype structs")))
	}

	///////////////////////////////////////////////////////////////////////////////
	// Deserialize compound types                                                //
	///////////////////////////////////////////////////////////////////////////////

	fn deserialize_tuple<V>(
		self,
		_len: usize,
		visitor: V,
	) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_any(visitor)
	}

	fn deserialize_tuple_struct<V>(
		self,
		_name: &'static str,
		len: usize,
		visitor: V,
	) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("Can't deserialize tuplle structs")))
	}

	fn deserialize_struct<V>(
		self,
		_name: &'static str,
		_fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_any(visitor)
	}

	fn deserialize_enum<V>(
		self,
		_name: &'static str,
		_variants: &'static [&'static str],
		_visitor: V,
	) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("Can't deserialize enums")))
	}
}

struct EpeeCompound<'a, 'de: 'a, R: Read> {
	deserializer: &'a mut Deserializer<'de, R>,
	remaining: usize,
	started: bool,
	size_hint: Option<usize>, // size hint provided at compile-time (used by structs & tuples)
	array_type: Option<EpeeScalarType>, // if == None, then this compound is a section,
	is_root: bool
}

impl<'de, 'a, R: Read> EpeeCompound<'a, 'de, R> {
	fn new_section(deserializer: &'a mut Deserializer<'de, R>, size_hint: Option<usize>) -> Self {
		Self {
			deserializer: deserializer,
			remaining: 0,
			started: false,
			size_hint: size_hint,
			array_type: None,
			is_root: false
		}
	}

	fn new_root_section(deserializer: &'a mut Deserializer<'de, R>, size_hint: Option<usize>) -> Self {
		Self {
			deserializer: deserializer,
			remaining: 0,
			started: false,
			size_hint: size_hint,
			array_type: None,
			is_root: true
		}
	}

	fn new_array(deserializer: &'a mut Deserializer<'de, R>, size_hint: Option<usize>, array_type: EpeeScalarType) -> Self {
		Self {
			deserializer: deserializer,
			remaining: 0,
			started: false,
			size_hint: size_hint,
			array_type: Some(array_type),
			is_root: false
		}
	}

	fn validate_signature(&mut self) -> Result<bool> {
		let mut sigbuf = [0u8; constants::PORTABLE_STORAGE_SIGNATURE_SIZE];
		self.deserializer.read_raw(&mut sigbuf)?;
		Ok(sigbuf == constants::PORTABLE_STORAGE_SIGNATURE)
	}

	fn start_if_necessary(&mut self) -> Result<()> {
		if self.started {
			return Ok(());
		}

		if self.is_root {
			let good_signature = self.validate_signature()?;
			if !good_signature {
				return epee_err!(ExpectedFormatSignature);
			}
		}

		// Get length from stream
		self.remaining = VarInt::from_reader(self.deserializer.reader)?.try_into()?;

		if let Some(size_hint) = self.size_hint {
			if size_hint != self.remaining {
				return epee_err!(SizeHintMismatch, "Deserialized length {} does not match size hint {}", self.remaining, size_hint);
			}
		}

		self.started = true;

		Ok(())
	}

	fn done(&self) -> bool {
		self.remaining == 0
	}
}

impl<'de, 'a, R: Read> SeqAccess<'de> for EpeeCompound<'a, 'de, R> {
	type Error = Error;

	// @TODO enforce that types are homogenous
	fn next_element_seed<T>(&mut self, seed: T) -> Result<Option<T::Value>>
	where
		T: DeserializeSeed<'de>
	{
		self.start_if_necessary()?;

		if self.done() {
			return Ok(None);
		}

		self.remaining -= 1;

		if let Some(array_type) = self.array_type {
			self.deserializer.state = DeserState::ExpectingScalar(array_type);
			let res = seed.deserialize(&mut *self.deserializer).map(Some);

			if self.done() {
				self.deserializer.state = DeserState::ExpectingKey;
			}

			res
		} else {
			epee_err!(CompoundMissingArrayType)
		}
	}
}

impl<'de, 'a, R: Read> MapAccess<'de> for EpeeCompound<'a, 'de, R> {
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
	where
		K: DeserializeSeed<'de>,
	{
		self.start_if_necessary()?;

		if self.done() {
			return Ok(None)
		}

		self.remaining -=1;

		self.deserializer.state = DeserState::ExpectingKey;
		let res = seed.deserialize(&mut *self.deserializer).map(Some);
		self.deserializer.state = DeserState::ExpectingEntry;

		res
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
	where
		V: DeserializeSeed<'de>,
	{
		self.deserializer.state = DeserState::ExpectingEntry;
		let res = seed.deserialize(&mut *self.deserializer);
		if self.is_root && self.remaining == 0 {
			self.deserializer.state = DeserState::Done;
		}
		res
	}
}