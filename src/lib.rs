pub mod de;
pub mod ser;
pub mod section;
pub mod constants;
pub mod error;
pub mod varint;

mod byte_counter;

// Conventional serde package structure
pub use de::{from_bytes, deserialize_from};
pub use error::{Error, Result, ErrorKind};
pub use ser::{to_bytes, serialize_into, serialized_size};

// EPEE-specific data types
pub use section::Section;
pub use varint::VarInt;