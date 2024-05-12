// Copyright Dan Burkert & Tokio Team - Apache 2 License
// <https://github.com/tokio-rs/prost/blob/26405ab8754219f101f489fe7bb2b74432b9b4b8/prost/src/encoding.rs>
//

//! Utility functions and types for encoding and decoding Protobuf types.
use core::str;
use core::u32;
use core::usize;

use ::bytes::BufMut;

/// Encodes an integer value into LEB128 variable length format, and writes it to the buffer.
/// The buffer must have enough remaining space (maximum 10 bytes).
#[inline]
pub fn encode_varint<B>(mut value: u64, buf: &mut B)
where
    B: BufMut,
{
    // Varints are never more than 10 bytes
    for _ in 0..10 {
        if value < 0x80 {
            buf.put_u8(value as u8);
            break;
        } else {
            buf.put_u8(((value & 0x7F) | 0x80) as u8);
            value >>= 7;
        }
    }
}

/// Returns the encoded length of the value in LEB128 variable length format.
/// The returned value will be between 1 and 10, inclusive.
#[inline]
pub fn encoded_len_varint(value: u64) -> usize {
    // Based on [VarintSize64][1].
    // [1]: https://github.com/google/protobuf/blob/3.3.x/src/google/protobuf/io/coded_stream.h#L1301-L1309
    ((((value | 1).leading_zeros() ^ 63) * 9 + 73) / 64) as usize
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
#[repr(u8)]
pub enum WireType {
    Varint = 0,
    SixtyFourBit = 1,
    LengthDelimited = 2,
    // StartGroup = 3,
    // EndGroup = 4,
    // ThirtyTwoBit = 5,
}

pub const MIN_TAG: u32 = 1;
pub const MAX_TAG: u32 = (1 << 29) - 1;

/// Encodes a Protobuf field key, which consists of a wire type designator and
/// the field tag.
#[inline]
pub fn encode_key<B>(tag: u32, wire_type: WireType, buf: &mut B)
where
    B: BufMut,
{
    debug_assert!((MIN_TAG..=MAX_TAG).contains(&tag));
    let key = (tag << 3) | wire_type as u32;
    encode_varint(u64::from(key), buf);
}

/// Returns the width of an encoded Protobuf field key with the given tag.
/// The returned width will be between 1 and 5 bytes (inclusive).
#[inline]
pub fn key_len(tag: u32) -> usize {
    encoded_len_varint(u64::from(tag << 3))
}

pub mod int32 {
    use crate::encoding::*;
    pub fn encode<B>(tag: u32, value: &i32, buf: &mut B)
    where
        B: BufMut,
    {
        encode_key(tag, WireType::Varint, buf);
        encode_varint(*value as u64, buf);
    }
}

pub mod double {
    use crate::encoding::*;
    pub fn encode<B>(tag: u32, value: &f64, buf: &mut B)
    where
        B: BufMut,
    {
        encode_key(tag, WireType::SixtyFourBit, buf);
        buf.put_f64_le(*value);
    }

    #[inline]
    pub fn encoded_len(tag: u32, _: &f64) -> usize {
        key_len(tag) + 8
    }
}

pub mod string {
    use super::*;

    pub fn encode<B>(tag: u32, value: &str, buf: &mut B)
    where
        B: BufMut,
    {
        encode_key(tag, WireType::LengthDelimited, buf);
        encode_varint(value.len() as u64, buf);
        buf.put_slice(value.as_bytes());
    }

    #[inline]
    pub fn encoded_len(tag: u32, value: &str) -> usize {
        key_len(tag) + encoded_len_varint(value.len() as u64) + value.len()
    }
}
