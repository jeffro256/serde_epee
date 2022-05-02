use std::io::Write;
use std::fs::File;

extern crate serde_epee;
use serde_epee::Section;

struct ExampleNested {
    double: f64,
    unsigned_64bit_int: u64
}

struct ExampleObject {
    short_quote: String,
    long_quote: String,
    signed_32bit_int: i32,
    array_of_bools: Vec<bool>,
    nested_section: ExampleNested
}



fn main() -> serde_epee::Result<()> {
    let example_data = ExampleObject {
        short_quote: "Give me liberty or give me death!".to_string(),
        long_quote: "Monero is more than just a technology. It's also what the technology stands for.".to_string(),
        signed_32bit_int: 20140418
        array_of_bools: vec![true, false, true, true],
        nested_section: ExampleNested {
            double: -6.9,
            unsigned_64bit_int: 11111111111111111111
        }
    }

    /*let mut section = Section::new();
    section.insert_u32("beep".to_string(), 47);
    section.insert_array_bool("ahhhhhhh".to_string(), vec![true, false, false, false, true, true]);

    let ser_bytes = serde_epee::to_bytes(&section)?;*/
    let mut outf = File::create("epee_example.dat")?;
    serde_epee::serialize_into(&mut outf, &example_data)
}
