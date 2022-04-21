pub mod de;
pub mod ser;
pub mod section;
pub mod constants;
pub mod error;
pub mod varint;

// Conventional serde structure
pub use de::{from_bytes, Deserializer};
pub use error::{Error, Result};
pub use ser::{to_bytes, Serializer};
pub use section::Section;
pub use varint::VarInt;