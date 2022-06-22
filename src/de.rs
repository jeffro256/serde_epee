// @TODO Make sure all deserialize_x functions take state into account state so type values are consumed

use std::io::Read;

use serde::Deserialize;
use serde::de::{self, Deserializer, DeserializeSeed, MapAccess, SeqAccess, Visitor};

use crate::constants;
use crate::error::{Error, ErrorKind, Result};
use crate::VarInt;

///////////////////////////////////////////////////////////////////////////////
// Error macros                                                              //
///////////////////////////////////////////////////////////////////////////////

macro_rules! err {
	($kind:ident) => (Err(Error::new_no_msg(ErrorKind::$kind)))
}

macro_rules! err_msg {
	($kind:ident, $fmt:expr, $($fmt_args:expr), *) => (
		Err(Error::new(ErrorKind::$kind, format!($fmt, $($fmt_args), *)))
	);
	($kind:ident, $msg:expr) => (
		Err(Error::new(ErrorKind::$kind, $msg.to_string()))
	)
}

///////////////////////////////////////////////////////////////////////////////
// User functions  (use these if you're new here)                            //
///////////////////////////////////////////////////////////////////////////////

pub fn from_reader<T, R>(mut reader: R) -> Result<T>
where
	T: de::DeserializeOwned,
	R: Read
{
	let mut deserializer = EpeeDeserializer::from_reader(&mut reader);
	T::deserialize(&mut deserializer)
}

pub fn from_bytes<'a, T>(bytes: &'a mut &[u8]) -> Result<T>
where
	T: Deserialize<'a>,
{
	let mut deserializer = EpeeDeserializer::from_reader(bytes);
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
			return err_msg!(BadTypeCode, "Invalid value: {}", type_code);
		}

		Ok(TYPES[scalar_type_code as usize - 1])
	}
}

trait AssociatedEpeeScalarType {
	fn associated_epee_scalar_type() -> EpeeScalarType;
}

macro_rules! impl_ass_scalar {
	( $type:ty, $epeevariant:expr ) => {
		impl AssociatedEpeeScalarType for $type {
			fn associated_epee_scalar_type() -> EpeeScalarType {
				$epeevariant
			}
		}
	}
}

impl_ass_scalar!{i64,    EpeeScalarType::Int64}
impl_ass_scalar!{i32,    EpeeScalarType::Int32}
impl_ass_scalar!{i16,    EpeeScalarType::Int16}
impl_ass_scalar!{i8,     EpeeScalarType::Int8}
impl_ass_scalar!{u64,    EpeeScalarType::UInt64}
impl_ass_scalar!{u32,    EpeeScalarType::UInt32}
impl_ass_scalar!{u16,    EpeeScalarType::UInt16}
impl_ass_scalar!{u8,     EpeeScalarType::UInt8}
impl_ass_scalar!{f64,    EpeeScalarType::Double}
impl_ass_scalar!{f32,    EpeeScalarType::Double}
impl_ass_scalar!{String, EpeeScalarType::Str}
impl_ass_scalar!{bool,   EpeeScalarType::Bool}

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

pub struct EpeeDeserializer<'de, R: Read> {
	reader: &'de mut R,
	state: DeserState,
}

impl<'de, R: Read> EpeeDeserializer<'de, R> {
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
			//Err(ioe) => Err(ioe.into())
			Err(ioe) => panic!("Error reading {} bytes", buf.len())
		}
	}

	fn read_single(&mut self) -> Result<u8> {
		let mut single_byte = [0u8];
		match self.reader.read_exact(&mut single_byte) {
			Ok(_) => Ok(single_byte[0]),
			Err(ioe) => Err(ioe.into())
		}
	}

	fn read_type_code(&mut self) -> Result<EpeeEntryType> {
		EpeeEntryType::from_type_code(self.read_single()?)
	}

	fn deserialize_section_entry<V>(&mut self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>
	{
		let entry_type = self.read_type_code()?;

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
				EpeeScalarType::Int64 => self.deserialize_i64(visitor),
				EpeeScalarType::Int32 => self.deserialize_i32(visitor),
				EpeeScalarType::Int16 => self.deserialize_i16(visitor),
				EpeeScalarType::Int8 => self.deserialize_i8(visitor),
				EpeeScalarType::UInt64 => self.deserialize_u64(visitor),
				EpeeScalarType::UInt32 => self.deserialize_u32(visitor),
				EpeeScalarType::UInt16 => self.deserialize_u16(visitor),
				EpeeScalarType::UInt8 => self.deserialize_u8(visitor),
				EpeeScalarType::Double => self.deserialize_f64(visitor),
				EpeeScalarType::Str => self.deserialize_str(visitor),
				EpeeScalarType::Bool => self.deserialize_bool(visitor),
				EpeeScalarType::Object => visitor.visit_map(EpeeCompound::new_section(self, None))
			}
		} else {
			err!(ExpectedScalar)
		}
	}
}

macro_rules! deserialize_num {
	( $sfname:ident, $vfname:ident, $numtype:ty ) => {
		fn $sfname<V>(self, visitor: V) -> Result<V::Value>
		where
			V: Visitor<'de>
		{
			if let DeserState::ExpectingEntry | DeserState::ExpectingScalar(_) = self.state {
				if let DeserState::ExpectingEntry = self.state {
					// consume and check type code
					let actual_type_code = self.read_type_code()?.scalar_type;
					let expected_type_code = <$numtype>::associated_epee_scalar_type();
					if actual_type_code != expected_type_code {
						return err_msg!(
							TypeMismatch,
							"Found type code for {:?} while trying to deserialize number type {:?}",
							actual_type_code,
							expected_type_code
						)
					}
				}

				const NBYTES: usize = std::mem::size_of::<$numtype>();
				let mut le_bytes = [0u8; NBYTES];
				self.read_raw(&mut le_bytes)?;
				let num = <$numtype>::from_le_bytes(le_bytes);
				println!("deser num {}", num);
				let visit_res = visitor.$vfname(num);
				self.state = DeserState::ExpectingEntry;
				visit_res	
			} else {
				return err_msg!(
					NotExpectingScalar,
					"Trying to deserialize {:?} while state is {:?}",
					<$numtype>::associated_epee_scalar_type(),
					self.state
				)
			}
		}
	}
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut EpeeDeserializer<'de, R> {
	type Error = Error;

	fn deserialize_any<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		match self.state {
			DeserState::ExpectingSection(true) => visitor.visit_map(EpeeCompound::new_root_section(self, None)),
			DeserState::ExpectingSection(false) => visitor.visit_map(EpeeCompound::new_section(self, None)),
			DeserState::ExpectingKey => self.deserialize_str(visitor),
			DeserState::ExpectingEntry => self.deserialize_section_entry(visitor),
			DeserState::ExpectingScalar(_) => self.deserialize_scalar(visitor),
			DeserState::Done => err_msg!(ExpectedEnd, "deserialize_any() was called after Deserializer was done")
		}
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let bool_byte = self.read_single()?;
		visitor.visit_bool(bool_byte != 0)
	}

	deserialize_num!{deserialize_u8,  visit_u8,  u8 }
	deserialize_num!{deserialize_u16, visit_u16, u16}
	deserialize_num!{deserialize_u32, visit_u32, u32}
	deserialize_num!{deserialize_u64, visit_u64, u64}
	deserialize_num!{deserialize_i8,  visit_i8,  i8 }
	deserialize_num!{deserialize_i16, visit_i16, i16}
	deserialize_num!{deserialize_i32, visit_i32, i32}
	deserialize_num!{deserialize_i64, visit_i64, i64}
	deserialize_num!{deserialize_f64, visit_f64, f64}

	fn deserialize_f32<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_f64(visitor) 
	}

	fn deserialize_char<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let mut scalar_buf = [0u8; 4];
		self.read_raw(&mut scalar_buf)?;
		let scalar_val = u32::from_le_bytes(scalar_buf);
		match scalar_val.try_into() {
			Ok(c) => visitor.visit_char(c),
			Err(_) => Err(Error::new_no_msg(ErrorKind::BadUnicodeScalar))
		}
	}

	fn deserialize_str<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		if let DeserState::ExpectingKey = self.state { // read string in as section key
			let mut strbuf = [0u8; 255];
			let strlen = self.read_single()? as usize;
			if strlen == 0 {
				return err_msg!(EmptySectionKey, "section key length can not be zero!");
			}
			let strslice = &mut strbuf[..strlen];
			self.read_raw(strslice)?;
			match std::str::from_utf8(strslice) {
				Ok(s) => visitor.visit_str(s),
				Err(_) => err_msg!(StringBadEncoding, "encoding error while expecting key on slice: {:?}", strslice)
			}
		} else if let DeserState::ExpectingEntry | DeserState::ExpectingScalar(_) = self.state { // Read as normal string
			if let DeserState::ExpectingEntry = self.state {
				// consume and check type code
				let actual_type_code = self.read_type_code()?.scalar_type;
				let expected_type_code = EpeeScalarType::Str;
				if actual_type_code != expected_type_code {
					return err_msg!(
						TypeMismatch,
						"Found type code for {:?} while trying to deserialize type {:?}",
						actual_type_code,
						expected_type_code
					)
				}
			}

			let varlen = VarInt::from_reader(self.reader)?;
			let strsize: usize = varlen.try_into()?;
			if strsize > constants::MAX_STRING_LEN_POSSIBLE {
				return Err(Error::new_no_msg(ErrorKind::StringTooLong))
			}

			let mut strbuf = vec![0u8; strsize];
			self.read_raw(strbuf.as_mut_slice())?;

			match std::str::from_utf8(strbuf.as_slice()) {
				Ok(s) => visitor.visit_str(s),
				Err(_) => visitor.visit_bytes(&strbuf)
			}
		} else {
			return err_msg!(
				NotExpectingScalar,
				"Trying to deserializes string while state is {:?}",
				self.state
			)
		}
	}

	fn deserialize_string<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_str(visitor)
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

	fn deserialize_seq<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		if let DeserState::ExpectingEntry = self.state {
			let entry_type = self.read_type_code()?;
			if entry_type.is_array {
				visitor.visit_seq(EpeeCompound::new_array(self, None, entry_type.scalar_type))
			} else {
				err_msg!(ExpectedArray, "Instead found {:?}", entry_type)
			}
		} else {
			err_msg!(NotExpectingArray, "but deserialize_seq() was called")
		}
	}

	fn deserialize_tuple<V>(self, _len: usize, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_seq(visitor)
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
		self.deserialize_tuple(len, visitor)
	}

	fn deserialize_map<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		if let DeserState::ExpectingSection(expecting_root) = self.state {
			if expecting_root {
				visitor.visit_map(EpeeCompound::new_root_section(self, None))
			} else {
				visitor.visit_map(EpeeCompound::new_section(self, None))
			}
		} else {
			err_msg!(NotExpectingSection, "In state {:?}, asked to deserialize map", self.state)
		}
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
		// @TODO specialize for structs
		self.deserialize_map(visitor)
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

	fn deserialize_identifier<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_str(visitor)
	}

	fn deserialize_ignored_any<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		self.deserialize_any(visitor)
	}
}

struct EpeeCompound<'a, 'de: 'a, R: Read> {
	deserializer: &'a mut EpeeDeserializer<'de, R>,
	remaining: usize,
	started: bool,
	size_hint: Option<usize>, // size hint provided at compile-time (used by structs & tuples)
	array_type: Option<EpeeScalarType>, // if == None, then this compound is a section,
	is_root: bool
}

impl<'de, 'a, R: Read> EpeeCompound<'a, 'de, R> {
	fn new_section(deserializer: &'a mut EpeeDeserializer<'de, R>, size_hint: Option<usize>) -> Self {
		Self {
			deserializer: deserializer,
			remaining: 0,
			started: false,
			size_hint: size_hint,
			array_type: None,
			is_root: false
		}
	}

	fn new_root_section(deserializer: &'a mut EpeeDeserializer<'de, R>, size_hint: Option<usize>) -> Self {
		Self {
			deserializer: deserializer,
			remaining: 0,
			started: false,
			size_hint: size_hint,
			array_type: None,
			is_root: true
		}
	}

	fn new_array(deserializer: &'a mut EpeeDeserializer<'de, R>, size_hint: Option<usize>, array_type: EpeeScalarType) -> Self {
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
				return err!(ExpectedFormatSignature);
			}
		}

		// Get length from stream
		self.remaining = VarInt::from_reader(self.deserializer.reader)?.try_into()?;

		if let Some(size_hint) = self.size_hint {
			if size_hint != self.remaining {
				return err_msg!(SizeHintMismatch, "Deserialized length {} does not match size hint {}", self.remaining, size_hint);
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
			err!(CompoundMissingArrayType)
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