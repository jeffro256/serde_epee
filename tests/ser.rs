use serde_epee;
use serde_epee::*;

use hex;
use serde::{Serialize, Deserialize};

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Serialize, Deserialize, Debug)]
    struct Request {
        txid: [u8; 32]
    }

    #[test]
    fn serialize_byte_array() {
        let expected_bytes_hex = "01110101010102010104047478696488801818181818181818181818181818181818181818181818181818181818181818";
        let expected_bytes_vec = hex::decode(expected_bytes_hex).unwrap();

        let foobar = Request { txid: [24; 32] };
        match serde_epee::to_bytes(&foobar) {
            Ok(foobytes) => assert_eq!(expected_bytes_vec, foobytes),
            Err(err) => panic!("Error: {}", err)
        }
    }
}
