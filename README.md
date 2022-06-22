# Epee {Ser/Deser}ialization Library in Pure Rust

This library aims to add simple and performant epee serialization in Rust for exisiting types with little to no extra boilerplate. You can serialize/deserialize any Rust type which implements the `Serialize` and `Deserialize` trait from the `serde` package, respectively. To learn more about the format specification, read [here](PORTABLE_STORAGE.md). Here are some examples:

## Serialization:

```Rust
use serde::{Serialize, Deserialize};
use serde_epee;

#[derive(Serialize, Deserialize, Debug)]
struct MyType {
    foo: u32,
    bar: i8,
    baz: String
}

fn main() {
    let foobar = MyType { foo: 23, bar: -1, baz: "Howdy, World!".to_string() };
    match serde_epee::to_bytes(&foobar) {
        Ok(foobytes) => println!("{:02x?}", foobytes),
        Err(err) => println!("Error: {}", err)
    }
}
```

This outputs the string:

`[01, 11, 01, 01, 01, 01, 02, 01, 01, 0c, 03, 66, 6f, 6f, 06, 17, 00, 00, 00, 03, 62, 61, 72, 04, ff, 03, 62, 61, 7a, 0a, 34, 48, 6f, 77, 64, 79, 2c, 20, 57, 6f, 72, 6c, 64, 21]`

You can also serialize directly to a `Write` interface:

```Rust
match File::create("epee_example.dat") {
    Ok(outf) => serde_epee::to_writer(outf, &foobar).unwrap(),
    Err(err) => println!("File error ;(")
}
```

## Deserialization

```Rust
use serde::{Serialize, Deserialize};
use serde_epee;

#[derive(Serialize, Deserialize, Debug)]
struct MyType {
    foo: u32,
    bar: i8,
    baz: String
}

fn main() {
    let example_bytes = [1u8, 17, 1, 1, 1, 1, 2, 1, 1, 12, 3, 102, 111, 111, 6, 23, 0, 0, 0, 3, 98, 97, 114, 4, 255, 3, 98, 97, 122, 10, 52, 72, 111, 119, 100, 121, 44, 32, 87, 111, 114, 108, 100, 33];

    let foobar = serde_epee::from_bytes(&examples_bytes).unwrap();
    println!("{:?}", foobar);
}
```
