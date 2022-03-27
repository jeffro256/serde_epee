extern crate serde_epee;

use serde_epee::{Result, Serializer, VarInt};

fn main() -> Result<()> {
    let mut moop = [0u8; 100];
    let s = Serializer::new(&mut moop[..]);
    let v = VarInt::from(55u64);
    let w: u8 = v.try_into()?;
    println!("Hello, world! {:?} {}", s, w);

    Ok(())
}
