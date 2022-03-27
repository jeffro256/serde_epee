mod de;
mod error;
mod ser;
mod varint;

// Conventional serde structure
//pub use de::{from_bytes, Deserializer};
pub use error::{Error, Result};
pub use ser::{/*to_bytes,*/ Serializer};

pub use varint::VarInt;