use std::io::Write;
use std::fs::File;
use std::time;

use serde::{Deserialize, Serialize};

extern crate serde_epee;
use serde_epee::Section;

#[derive(Deserialize, Serialize, Debug)]
struct NodeData {
    local_time: u64,
    my_port: u32,
    network_id: String,
    peer_id: u64
}

#[derive(Deserialize, Serialize, Debug)]
struct PayloadData {
    cumulative_difficulty: u64,
    current_height: u64,
    //top_id: &'static str,
    #[serde(with = "serde_bytes")]
    top_id: Vec<u8>,
    top_version: u8
}

#[derive(Deserialize, Serialize, Debug)]
struct HandshakeSection {
    node_data: NodeData,
    payload_data: PayloadData
}

#[derive(Deserialize, Serialize, Debug)]
struct TestStruct {
	var_a: u32,
	var_b: u8
}

fn unix_now() -> u64 {
    time::SystemTime::now().duration_since(time::UNIX_EPOCH).unwrap().as_secs()
}

fn create_handshake() -> HandshakeSection {
    let node_data = NodeData {
        local_time: unix_now(),
        my_port: 0,
        network_id: String::new(),
        peer_id: 2412070617452275018 // "Jeffrey!"
    };

    let payload_data = PayloadData {
        cumulative_difficulty: 1,
        current_height: 1,
        top_id: vec![65, 128, 21, 187, 154, 233, 130, 161, 151, 93, 167, 215, 146, 119, 194, 112, 87, 39, 165, 104, 148, 186, 15, 178, 70, 173, 170, 187, 31, 70, 50, 227],
        top_version: 1
    };

    HandshakeSection { node_data: node_data, payload_data: payload_data }
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

    /*
	let mut inf = File::open("ser_test.dat")?;
	let mut de = serde_epee::Deserializer::from_reader(&mut inf)?;
    let v = TestStruct::deserialize(&mut de)?;
	println!("{:?}", v);
    */

    let mut section = Section::new();
    section.insert_u32("beep".to_string(), 47);
    section.insert_array_bool("ahhhhhhh".to_string(), vec![true, false, false, false, true, true]);

    //let handshake = create_handshake();
    let ser_bytes = serde_epee::to_bytes(&section)?;
    let mut outf = File::create("section_test.dat")?;
    outf.write_all(&ser_bytes)?;

	Ok(())
}
