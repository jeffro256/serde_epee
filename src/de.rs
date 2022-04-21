use std::io::Read;

use serde::Deserialize;
use serde::de::{self, DeserializeSeed, MapAccess, SeqAccess, Visitor};

use crate::constants;
use crate::error::{Error, ErrorKind, Result};
use crate::VarInt;

///////////////////////////////////////////////////////////////////////////////
// User functions                                                            //
///////////////////////////////////////////////////////////////////////////////

pub fn deserialize_from<'a, T, R>(reader: &'a mut R) -> Result<T>
where
	T: Deserialize<'a>,
	R: Read
{
	let mut deserializer = Deserializer::from_reader(reader)?;
	T::deserialize(&mut deserializer)
}

// @TODO: Look into implementing Deserializer for DeserializeOwned to reduce copies
pub fn from_bytes<'a, T>(bytes: &'a mut &[u8]) -> Result<T>
where
	T: Deserialize<'a>,
{
	let mut deserializer = Deserializer::from_reader(bytes)?;
	T::deserialize(&mut deserializer)
}

///////////////////////////////////////////////////////////////////////////////
// Deserializer definition                                                   //
///////////////////////////////////////////////////////////////////////////////

pub struct Deserializer<'de, R: Read> {
	reader: &'de mut R,
	deserializing_key: bool
}

impl<'de, R: Read> Deserializer<'de, R> {
	///////////////////////////////////////////////////////////////////////////////
	// Constructors                                                              //
	///////////////////////////////////////////////////////////////////////////////
	pub fn from_reader(reader: &'de mut R) -> Result<Self> {
		let mut res = Deserializer { reader: reader, deserializing_key: false };
		if res.validate_signature()? {
			Ok(res)
		} else {
			Err(Error::new_no_msg(ErrorKind::ExpectedFormatSignature))
		}
	}

	///////////////////////////////////////////////////////////////////////////////
	// Reading methods                                                           //
	///////////////////////////////////////////////////////////////////////////////

	fn read_raw(&mut self, buf: &mut [u8]) -> Result<()> {
		let read_res = self.reader.read_exact(buf);
		match read_res { 
			Ok(_) => Ok(()),
			Err(ioe) => Err(ioe.into())
		}
	}

	fn read_single(&mut self) -> Result<u8> {
		let mut single_byte = [0u8];
		match self.reader.read_exact(&mut single_byte) {
			Ok(_) => Ok(single_byte[0]),
			Err(ioe) => Err(ioe.into())
		}
	}

	fn read_type_code(&mut self) -> Result<(u8, bool)> {
		let full_type_code = self.read_single()?; 
		let is_array = (full_type_code & constants::SERIALIZE_FLAG_ARRAY) != 0;
		let type_code = full_type_code & !constants::SERIALIZE_FLAG_ARRAY; // unset array bit

		if type_code >= constants::SERIALIZE_TYPE_INT64 && type_code <= constants::SERIALIZE_TYPE_OBJECT {
			Ok((type_code, is_array))
		} else {
			Err(Error::new_no_msg(ErrorKind::BadTypeCode))
		}
	}

	fn validate_signature(&mut self) -> Result<bool> {
		let mut sigbuf = [0u8; constants::PORTABLE_STORAGE_SIGNATURE_SIZE];
		self.read_raw(&mut sigbuf)?;
		Ok(sigbuf == constants::PORTABLE_STORAGE_SIGNATURE)
	}
}

macro_rules! deserialize_num {
	( $sfname:ident, $vfname:ident, $numtype:ty, $numsize:expr ) => {
		fn $sfname<V>(self, visitor: V) -> Result<V::Value>
		where
			V: Visitor<'de>
		{
			let mut le_bytes = [0u8; $numsize];
			self.read_raw(&mut le_bytes)?;
			visitor.$vfname(<$numtype>::from_le_bytes(le_bytes))
		}
	}
}

impl<'de, 'a, R: Read> de::Deserializer<'de> for &'a mut Deserializer<'de, R> {
	type Error = Error;

	fn deserialize_any<V>(self, _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		unimplemented!()
	}

	fn deserialize_bool<V>(self, visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		let bool_byte = self.read_single()?;
		visitor.visit_bool(bool_byte != 0)
	}

	deserialize_num!{deserialize_u8,  visit_u8,  u8,  1}
	deserialize_num!{deserialize_u16, visit_u16, u16, 2}
	deserialize_num!{deserialize_u32, visit_u32, u32, 4}
	deserialize_num!{deserialize_u64, visit_u64, u64, 8}
	deserialize_num!{deserialize_i8,  visit_i8,  i8,  1}
	deserialize_num!{deserialize_i16, visit_i16, i16, 2}
	deserialize_num!{deserialize_i32, visit_i32, i32, 4}
	deserialize_num!{deserialize_i64, visit_i64, i64, 8}
	deserialize_num!{deserialize_f64, visit_f64, f64, 8}

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
		if self.deserializing_key { // read string in as section key
			let mut strbuf = [0u8; 255];
			let strlen = self.read_single()? as usize;
			println!("{}", strlen);
			let strslice = &mut strbuf[..strlen];
			self.read_raw(strslice)?;
			println!("{:?}", strslice);
			match std::str::from_utf8(strslice) {
				Ok(s) => visitor.visit_str(s),
				Err(_) => Err(Error::new_no_msg(ErrorKind::StringBadEncoding))
			}
		} else { // Read as normal string
			let varlen = VarInt::from_reader(self.reader)?;
			let strsize: usize = varlen.try_into()?;
			if strsize > constants::MAX_STRING_LEN_POSSIBLE {
				return Err(Error::new_no_msg(ErrorKind::StringTooLong))
			}

			let mut strbuf = vec![0u8; strsize];
			self.read_raw(strbuf.as_mut_slice())?;

			match std::str::from_utf8(strbuf.as_slice()) {
				Ok(s) => visitor.visit_str(s),
				Err(_) => Err(Error::new_no_msg(ErrorKind::StringBadEncoding))
			}
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

	fn deserialize_unit<V>(self, _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("Can't deserialize anonymous units")))
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
		let (_element_type_code, is_array) = self.read_type_code()?;

		if !is_array {
			return Err(Error::new_no_msg(ErrorKind::ExpectedArray));
		}

		let array_varlen = VarInt::from_reader(self.reader)?;
		let array_size: usize = array_varlen.try_into()?;

		if array_size > constants::MAX_SECTION_KEY_SIZE {
			return Err(Error::new_no_msg(ErrorKind::ArrayTooLong));
		}

		let seq_de = EpeeCompound::new_array(self, array_size); 
		let value = visitor.visit_seq(seq_de)?;
		// @TODO Check if sequence is done
		/*
		if seq_de.done() {
			Ok(value)
		} else {
			Err(Error::new_no_msg(ErrorKind::ExpectedArrayEnd))
		}
		*/

		Ok(value)
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

	fn deserialize_map<V>(self, _visitor: V) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		Err(Error::new(ErrorKind::SerdeModelUnsupported, String::from("Can't deserialize maps")))
	}

	fn deserialize_struct<V>(
		self,
		_name: &'static str,
		fields: &'static [&'static str],
		visitor: V,
	) -> Result<V::Value>
	where
		V: Visitor<'de>,
	{
		visitor.visit_map(EpeeCompound::new_section(self, fields.len()))
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
	deserializer: &'a mut Deserializer<'de, R>,
	remaining: usize,
	//type_code: u8
}

impl<'de, 'a, R: Read> EpeeCompound<'a, 'de, R> {
	fn new_array(deserializer: &'a mut Deserializer<'de, R>, array_len: usize) -> Self {
		Self {
			deserializer: deserializer,
			remaining: array_len,
			//type_code: type_code
		}
	}

	fn new_section(deserializer: &'a mut Deserializer<'de, R>, section_len: usize) -> Self {
		Self {
			deserializer: deserializer,
			remaining: section_len,
			//type_code: constants::SERIALIZE_TYPE_UNKNOWN
		}
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
		if self.done() {
			return Ok(None);
		}

		self.remaining -= 1;

		seed.deserialize(&mut *self.deserializer).map(Some)
	}
}

impl<'de, 'a, R: Read> MapAccess<'de> for EpeeCompound<'a, 'de, R> {
	type Error = Error;

	fn next_key_seed<K>(&mut self, seed: K) -> Result<Option<K::Value>>
	where
		K: DeserializeSeed<'de>,
	{
		if self.done() {
			return Ok(None)
		}

		self.remaining -=1;

		self.deserializer.deserializing_key = true;
		seed.deserialize(&mut *self.deserializer).map(Some)
	}

	fn next_value_seed<V>(&mut self, seed: V) -> Result<V::Value>
	where
		V: DeserializeSeed<'de>,
	{
		self.deserializer.deserializing_key = false;
		self.deserializer.read_single()?; // consume type code
		seed.deserialize(&mut *self.deserializer)
	}
}