//! MessagePack exactly as mymind's front-end codec (`@msgpack/msgpack` 3.1.1, mirrored as `vendor/msgpack.min.js`)
//! writes and reads it with its default extension codec, plus the replica's own encoder for recorded cards.

mod decode;
mod encode;
mod recorded;
mod timestamp;

pub use decode::{DecodeError, DecodeOptions, decode, decode_multi};
pub use encode::{EncodeError, EncodeOptions, encode, encode_into};
pub use recorded::encode_recorded;
pub use timestamp::{EXT_TIMESTAMP, date_from_timestamp, timestamp, timestamp_from_date};
