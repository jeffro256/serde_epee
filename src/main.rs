extern crate serde_epee;

use std::io::Write;
use std::fs::File;

use serde::Serialize;

#[derive(Serialize)]
struct TestStruct {
    var_a: u32,
    var_b: u8
}

use serde_epee::{Result, to_bytes};

fn main() -> Result<()> {
    let test_val = TestStruct { var_a: 4242, var_b: 77 };
    let b = to_bytes(&test_val)?;
    let mut f = File::create("ser_test.dat")?;
    f.write(&b)?;

    Ok(())
}
