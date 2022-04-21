use std::io::Write;
use std::fs::File;

extern crate serde_epee;
use serde_epee::Section;

fn main() -> serde_epee::Result<()> {
    let mut section = Section::new();
    section.insert_u32("beep".to_string(), 47);
    section.insert_array_bool("ahhhhhhh".to_string(), vec![true, false, false, false, true, true]);

    let ser_bytes = serde_epee::to_bytes(&section)?;
    let mut outf = File::create("section_test.dat")?;
    outf.write_all(&ser_bytes)?;

	Ok(())
}
