use std::collections::HashMap;

use serde;
use serde::Serialize;
use serde_bytes;

#[derive(Clone, Serialize)]
#[serde(untagged)]
enum SectionScalar {
	Int64(i64),
	Int32(i32),
	Int16(i16),
	Int8(i8),
	UInt64(u64),
	UInt32(u32),
	UInt16(u16),
	UInt8(u8),
	Double(f64),
	Blob(serde_bytes::ByteBuf),
	// Blob and Str serialize to the same thing, Str is provided for convenience if encodeable as UTF-8
	Str(String), 
	Bool(bool),
	Object(Section)
}

// The reason for the difference between "scalars" and "arrays" is because EPEE doesn't allow nested arrays
#[derive(Clone, Serialize)]
#[serde(untagged)]
enum SectionArray {
	Int64(Vec<i64>),
	Int32(Vec<i32>),
	Int16(Vec<i16>),
	Int8(Vec<i8>),
	UInt64(Vec<u64>),
	UInt32(Vec<u32>),
	UInt16(Vec<u16>),
	UInt8(Vec<u8>),
	Double(Vec<f64>),
	Blob(Vec<serde_bytes::ByteBuf>),
	// Blob and Str serialize to the same thing, Str is provided for convenience if encodeable as UTF-8
	Str(Vec<String>), 
	Bool(Vec<bool>),
	Object(Vec<Section>)
}

#[derive(Clone, Serialize)]
#[serde(untagged)]
enum SectionEntry {
	Scalar(SectionScalar),
	Array(SectionArray)
}

#[derive(Clone, Serialize)]
#[serde(transparent)]
pub struct Section(HashMap<String, SectionEntry>);

// Returns Some(T) if an entry contains a scalar of the correct variant, $sname. None otherwise.
macro_rules! entry_shed_scalar {
	($entry:expr, $sname:ident) => (
		if let SectionEntry::Scalar(SectionScalar::$sname(ref val)) = $entry {
			Some(val)
		} else {
			None
		}
	)
}

// Returns Some(T) if an entry contains an array of the correct variant, $sname. None otherwise.
macro_rules! entry_shed_array {
	($entry:expr, $sname:ident) => (
		if let SectionEntry::Array(SectionArray::$sname(ref val)) = $entry {
			Some(val)
		} else {
			None
		}
	)
}

macro_rules! insert_scalar {
	($stype:ty, $fname:ident, $sname:ident) => {
		pub fn $fname(&mut self, k: String, v: $stype) -> Option<$stype> {
			let insert_res = self.0.insert(k, SectionEntry::Scalar(SectionScalar::$sname(v)));
			if let Some(SectionEntry::Scalar(SectionScalar::$sname(val))) = insert_res {
				Some(val)
			} else {
				None
			}
		}
	}
}

// @TODO: Return array replaced by insert as Option<Vec<$stype>>
macro_rules! insert_array {
	($stype:ty, $fname:ident, $sname:ident) => {
		pub fn $fname(&mut self, k: String, arr: Vec<$stype>){
			self.0.insert(k, SectionEntry::Array(SectionArray::$sname(arr)));
		}
	}
}

macro_rules! get_scalar {
	($stype:ty, $fname:ident, $sname:ident) => {
		pub fn $fname(&mut self, k: &String) -> Option<&$stype> {
			let get_res = self.0.get(k);
			if let Some(entry) = get_res {
				entry_shed_scalar!(entry, $sname)
			} else {
				None
			}
		}
	}
}

macro_rules! get_array {
	($stype:ty, $fname:ident, $sname:ident) => {
		pub fn $fname(&mut self, k: &String) -> Option<&Vec<$stype>> {
			let get_res = self.0.get(k);
			if let Some(entry) = get_res {
				entry_shed_array!(entry, $sname)
			} else {
				None
			}
		}
	}
}

impl Section {
	pub fn new() -> Self {
		Self( HashMap::<String, SectionEntry>::new() )
	}

	insert_scalar!{i64, insert_i64, Int64}
	insert_scalar!{i32, insert_i32, Int32}
	insert_scalar!{i16, insert_i16, Int16}
	insert_scalar!{i8, insert_i8, Int8}
	insert_scalar!{u64, insert_u64, UInt64}
	insert_scalar!{u32, insert_u32, UInt32}
	insert_scalar!{u16, insert_u16, UInt16}
	insert_scalar!{u8, insert_u8, UInt8}
	insert_scalar!{f64, insert_double, Double}
	insert_scalar!{serde_bytes::ByteBuf, insert_blob, Blob}
	insert_scalar!{String, insert_string, Str}
	insert_scalar!{bool, insert_bool, Bool}
	insert_scalar!{Section, insert_section, Object}

	insert_array!{i64, insert_array_i64, Int64}
	insert_array!{i32, insert_array_i32, Int32}
	insert_array!{i16, insert_array_i16, Int16}
	insert_array!{i8, insert_array_i8, Int8}
	insert_array!{u64, insert_array_u64, UInt64}
	insert_array!{u32, insert_array_u32, UInt32}
	insert_array!{u16, insert_array_u16, UInt16}
	insert_array!{u8, insert_array_u8, UInt8}
	insert_array!{f64, insert_array_double, Double}
	insert_array!{serde_bytes::ByteBuf, insert_blob_string, Blob}
	insert_array!{String, insert_array_string, Str}
	insert_array!{bool, insert_array_bool, Bool}
	insert_array!{Section, insert_array_section, Object}

	get_scalar!{i64, get_i64, Int64}
	get_scalar!{i32, get_i32, Int32}
	get_scalar!{i16, get_i16, Int16}
	get_scalar!{i8, get_i8, Int8}
	get_scalar!{u64, get_u64, UInt64}
	get_scalar!{u32, get_u32, UInt32}
	get_scalar!{u16, get_u16, UInt16}
	get_scalar!{u8, get_u8, UInt8}
	get_scalar!{f64, get_double, Double}
	get_scalar!{serde_bytes::ByteBuf, get_blob, Blob}
	get_scalar!{String, get_string, Str}
	get_scalar!{bool, get_bool, Bool}
	get_scalar!{Section, get_section, Object}

	get_array!{i64, get_array_i64, Int64}
	get_array!{i32, get_array_i32, Int32}
	get_array!{i16, get_array_i16, Int16}
	get_array!{i8, get_array_i8, Int8}
	get_array!{u64, get_array_u64, UInt64}
	get_array!{u32, get_array_u32, UInt32}
	get_array!{u16, get_array_u16, UInt16}
	get_array!{u8, get_array_u8, UInt8}
	get_array!{f64, get_array_double, Double}
	get_array!{serde_bytes::ByteBuf, get_array_blob, Blob}
	get_array!{String, get_array_string, Str}
	get_array!{bool, get_array_bool, Bool}
	get_array!{Section, get_array_section, Object}


	pub fn remove(&mut self, k: &String) {
		self.0.remove(k);
	}
}
