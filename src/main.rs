use std::io::{Read, Write};
use std::fs::File;

extern crate serde_epee;
use serde_epee::Section;
use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
struct ExampleNested {
    double: f64,
    unsigned_64bit_int: u64
}

#[derive(Debug, Serialize, Deserialize)]
struct ExampleObject {
    short_quote: String,
    long_quote: String,
    signed_32bit_int: i32,
    array_of_bools: Vec<bool>,
    nested_section: ExampleNested
}

fn main() -> serde_epee::Result<()> {
    let inf = File::open("epee_example.dat")?;
    let section: Section = serde_epee::from_reader(inf)?;

    println!("Result: {:?}", section);

    Ok(())
}
