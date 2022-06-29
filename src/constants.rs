pub const PORTABLE_STORAGE_SIGNATUREA: u32 = 0x01011101;
pub const PORTABLE_STORAGE_SIGNATUREB: u32 = 0x01020101;
pub const PORTABLE_STORAGE_FORMAT_VER: u8 = 0x01;
pub const PORTABLE_STORAGE_SIGNATURE_SIZE: usize = 9;
pub const PORTABLE_STORAGE_SIGNATURE: [u8; PORTABLE_STORAGE_SIGNATURE_SIZE] = [0x01, 0x11, 0x01, 0x01, 0x01, 0x01, 0x02, 0x01, 0x01];

pub const SERIALIZE_TYPE_UNKNOWN:u8 =       0;
pub const SERIALIZE_TYPE_INT64  :u8 =       1;
pub const SERIALIZE_TYPE_INT32  :u8 =       2;
pub const SERIALIZE_TYPE_INT16  :u8 =       3;
pub const SERIALIZE_TYPE_INT8   :u8 =       4;
pub const SERIALIZE_TYPE_UINT64 :u8 =       5;
pub const SERIALIZE_TYPE_UINT32 :u8 =       6;
pub const SERIALIZE_TYPE_UINT16 :u8 =       7;
pub const SERIALIZE_TYPE_UINT8  :u8 =       8;
pub const SERIALIZE_TYPE_DOUBLE :u8 =       9;
pub const SERIALIZE_TYPE_STRING :u8 =      10;
pub const SERIALIZE_TYPE_BOOL   :u8 =      11;
pub const SERIALIZE_TYPE_OBJECT :u8 =      12;
//pub const SERIALIZE_TYPE_ARRAY  :u8 =      13; // Currently unimplemented in library

pub const SERIALIZE_FLAG_ARRAY  :u8 =    0x80;

pub const MAX_NUM_SECTION_FIELDS:usize = 10000; // I made this limit up, not related to Monero/EPEE
pub const MAX_SECTION_KEY_SIZE:  usize =  255;
pub const MAX_STRING_LEN_POSSIBLE:usize = 2000000000; // "do not let string be so big"
pub const MAX_STRING_BUFFER_SIZE:usize = 4096; // In order to prevent memory allocation spam