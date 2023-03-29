use std::collections::HashMap;

use serde;
use serde::{Serialize, Deserialize};
use serde_bytes;

// The reason for a special array variant is that EPEE doesn't allow immediately nested arrays
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SectionArray {
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
	Bool(Vec<bool>),
	Object(Vec<Section>)
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum SectionEntry {
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
	Bool(bool),
	Object(Section),
	Array(SectionArray)
}

pub type Section = HashMap<String, SectionEntry>;
