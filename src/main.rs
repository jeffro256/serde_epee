use std::io::Write;
use std::fs::File;

use serde::{Deserialize, Serialize};

extern crate serde_epee;
//use serde_epee;
//use serde_epee::{Result, to_bytes, Deserializer, VarInt};

#[derive(Deserialize, Serialize, Debug)]
struct TestStruct {
	var_a: u32,
	var_b: u8
}

fn main() -> serde_epee::Result<()> {
	/*
	let test_val = TestStruct { var_a: 4242, var_b: 77 };
	let b = to_bytes(&test_val)?;
	let mut f = File::create("ser_test.dat")?;
	f.write(&b)?;

	let mut varf = File::create("var_test.dat")?;
	let varint = VarInt::try_from(4000000000u64).unwrap();
	varint.to_writer(&mut varf)?;
	varf.sync_all()?;
	*/

	let mut inf = File::open("ser_test.dat")?;
	let mut de = serde_epee::Deserializer::from_reader(&mut inf)?;
    let v = TestStruct::deserialize(&mut de)?;
	println!("{:?}", v);

	Ok(())
}
