// Copyright Dan Burkert & Tokio Team - Apache 2 License
// <https://github.com/tokio-rs/prost/blob/26405ab8754219f101f489fe7bb2b74432b9b4b8/prost/src/encoding.rs>
//

//! Utility functions and types for encoding and decoding Protobuf types.
//!
//! Meant to be used only from `Message` implementations.

#![allow(clippy::implicit_hasher, clippy::ptr_arg)]

use core::cmp::min;
use core::mem;
use core::str;
use core::u32;
use core::usize;
use std::collections::BTreeMap;
use std::format;
use std::string::String;
use std::vec::Vec;

use ::bytes::{Buf, BufMut, Bytes};

// use crate::DecodeError;
// use crate::Message;

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
    StartGroup = 3,
    EndGroup = 4,
    ThirtyTwoBit = 5,
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

// /// Helper macro which emits an `encode_repeated` function for the type.
// macro_rules! encode_repeated {
//     ($ty:ty) => {
//         pub fn encode_repeated<B>(tag: u32, values: &[$ty], buf: &mut B)
//         where
//             B: BufMut,
//         {
//             for value in values {
//                 encode(tag, value, buf);
//             }
//         }
//     };
// }

/// Macro which emits a module containing a set of encoding functions for a
/// variable width numeric type.
macro_rules! varint {
    ($ty:ty,
     $proto_ty:ident) => (
        varint!($ty,
                $proto_ty,
                to_uint64(value) { *value as u64 },
                from_uint64(value) { value as $ty });
    );

    ($ty:ty,
     $proto_ty:ident,
     to_uint64($to_uint64_value:ident) $to_uint64:expr,
     from_uint64($from_uint64_value:ident) $from_uint64:expr) => (

         pub mod $proto_ty {
            use crate::encoding::*;

            pub fn encode<B>(tag: u32, $to_uint64_value: &$ty, buf: &mut B) where B: BufMut {
                encode_key(tag, WireType::Varint, buf);
                encode_varint($to_uint64, buf);
            }

            // encode_repeated!($ty);

            pub fn encode_packed<B>(tag: u32, values: &[$ty], buf: &mut B) where B: BufMut {
                if values.is_empty() { return; }

                encode_key(tag, WireType::LengthDelimited, buf);
                let len: usize = values.iter().map(|$to_uint64_value| {
                    encoded_len_varint($to_uint64)
                }).sum();
                encode_varint(len as u64, buf);

                for $to_uint64_value in values {
                    encode_varint($to_uint64, buf);
                }
            }

            #[inline]
            pub fn encoded_len(tag: u32, $to_uint64_value: &$ty) -> usize {
                key_len(tag) + encoded_len_varint($to_uint64)
            }

            #[inline]
            pub fn encoded_len_repeated(tag: u32, values: &[$ty]) -> usize {
                key_len(tag) * values.len() + values.iter().map(|$to_uint64_value| {
                    encoded_len_varint($to_uint64)
                }).sum::<usize>()
            }

            #[inline]
            pub fn encoded_len_packed(tag: u32, values: &[$ty]) -> usize {
                if values.is_empty() {
                    0
                } else {
                    let len = values.iter()
                                    .map(|$to_uint64_value| encoded_len_varint($to_uint64))
                                    .sum::<usize>();
                    key_len(tag) + encoded_len_varint(len as u64) + len
                }
            }
        }
    );
}
varint!(bool, bool,
        to_uint64(value) u64::from(*value),
        from_uint64(value) value != 0);
varint!(i32, int32);
varint!(i64, int64);
varint!(u32, uint32);
varint!(u64, uint64);
varint!(i32, sint32,
to_uint64(value) {
    ((value << 1) ^ (value >> 31)) as u32 as u64
},
from_uint64(value) {
    let value = value as u32;
    ((value >> 1) as i32) ^ (-((value & 1) as i32))
});
varint!(i64, sint64,
to_uint64(value) {
    ((value << 1) ^ (value >> 63)) as u64
},
from_uint64(value) {
    ((value >> 1) as i64) ^ (-((value & 1) as i64))
});

/// Macro which emits a module containing a set of encoding functions for a
/// fixed width numeric type.
macro_rules! fixed_width {
    ($ty:ty,
     $width:expr,
     $wire_type:expr,
     $proto_ty:ident,
     $put:ident,
     $get:ident) => {
        pub mod $proto_ty {
            use crate::encoding::*;

            pub fn encode<B>(tag: u32, value: &$ty, buf: &mut B)
            where
                B: BufMut,
            {
                encode_key(tag, $wire_type, buf);
                buf.$put(*value);
            }

            // encode_repeated!($ty);

            pub fn encode_packed<B>(tag: u32, values: &[$ty], buf: &mut B)
            where
                B: BufMut,
            {
                if values.is_empty() {
                    return;
                }

                encode_key(tag, WireType::LengthDelimited, buf);
                let len = values.len() as u64 * $width;
                encode_varint(len as u64, buf);

                for value in values {
                    buf.$put(*value);
                }
            }

            #[inline]
            pub fn encoded_len(tag: u32, _: &$ty) -> usize {
                key_len(tag) + $width
            }

            #[inline]
            pub fn encoded_len_repeated(tag: u32, values: &[$ty]) -> usize {
                (key_len(tag) + $width) * values.len()
            }

            #[inline]
            pub fn encoded_len_packed(tag: u32, values: &[$ty]) -> usize {
                if values.is_empty() {
                    0
                } else {
                    let len = $width * values.len();
                    key_len(tag) + encoded_len_varint(len as u64) + len
                }
            }
        }
    };
}
fixed_width!(
    f32,
    4,
    WireType::ThirtyTwoBit,
    float,
    put_f32_le,
    get_f32_le
);
fixed_width!(
    f64,
    8,
    WireType::SixtyFourBit,
    double,
    put_f64_le,
    get_f64_le
);
fixed_width!(
    u32,
    4,
    WireType::ThirtyTwoBit,
    fixed32,
    put_u32_le,
    get_u32_le
);
fixed_width!(
    u64,
    8,
    WireType::SixtyFourBit,
    fixed64,
    put_u64_le,
    get_u64_le
);
fixed_width!(
    i32,
    4,
    WireType::ThirtyTwoBit,
    sfixed32,
    put_i32_le,
    get_i32_le
);
fixed_width!(
    i64,
    8,
    WireType::SixtyFourBit,
    sfixed64,
    put_i64_le,
    get_i64_le
);

/// Macro which emits encoding functions for a length-delimited type.
macro_rules! length_delimited {
    ($ty:ty) => {
        // encode_repeated!($ty);

        #[inline]
        pub fn encoded_len(tag: u32, value: &$ty) -> usize {
            key_len(tag) + encoded_len_varint(value.len() as u64) + value.len()
        }

        // #[inline]
        // pub fn encoded_len_repeated(tag: u32, values: &[$ty]) -> usize {
        //     key_len(tag) * values.len()
        //         + values
        //             .iter()
        //             .map(|value| encoded_len_varint(value.len() as u64) + value.len())
        //             .sum::<usize>()
        // }
    };
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

    length_delimited!(str);
}

pub trait BytesAdapter: sealed::BytesAdapter {}

mod sealed {
    use super::{Buf, BufMut};

    pub trait BytesAdapter: Default + Sized + 'static {
        fn len(&self) -> usize;

        /// Replace contents of this buffer with the contents of another buffer.
        fn replace_with<B>(&mut self, buf: B)
        where
            B: Buf;

        /// Appends this buffer to the (contents of) other buffer.
        fn append_to<B>(&self, buf: &mut B)
        where
            B: BufMut;

        fn is_empty(&self) -> bool {
            self.len() == 0
        }
    }
}

impl BytesAdapter for Bytes {}

impl sealed::BytesAdapter for Bytes {
    fn len(&self) -> usize {
        Buf::remaining(self)
    }

    fn replace_with<B>(&mut self, mut buf: B)
    where
        B: Buf,
    {
        *self = buf.copy_to_bytes(buf.remaining());
    }

    fn append_to<B>(&self, buf: &mut B)
    where
        B: BufMut,
    {
        buf.put(self.clone())
    }
}

impl BytesAdapter for Vec<u8> {}

impl sealed::BytesAdapter for Vec<u8> {
    fn len(&self) -> usize {
        Vec::len(self)
    }

    fn replace_with<B>(&mut self, buf: B)
    where
        B: Buf,
    {
        self.clear();
        self.reserve(buf.remaining());
        self.put(buf);
    }

    fn append_to<B>(&self, buf: &mut B)
    where
        B: BufMut,
    {
        buf.put(self.as_slice())
    }
}

pub mod bytes {
    use super::*;

    pub fn encode<A, B>(tag: u32, value: &A, buf: &mut B)
    where
        A: BytesAdapter,
        B: BufMut,
    {
        encode_key(tag, WireType::LengthDelimited, buf);
        encode_varint(value.len() as u64, buf);
        value.append_to(buf);
    }

    length_delimited!(impl BytesAdapter);
}

// pub mod message {
//     use super::*;

//     pub fn encode<M, B>(tag: u32, msg: &M, buf: &mut B)
//     where
//         M: Message,
//         B: BufMut,
//     {
//         encode_key(tag, WireType::LengthDelimited, buf);
//         encode_varint(msg.encoded_len() as u64, buf);
//         msg.encode_raw(buf);
//     }

//     pub fn encode_repeated<M, B>(tag: u32, messages: &[M], buf: &mut B)
//     where
//         M: Message,
//         B: BufMut,
//     {
//         for msg in messages {
//             encode(tag, msg, buf);
//         }
//     }

//     #[inline]
//     pub fn encoded_len<M>(tag: u32, msg: &M) -> usize
//     where
//         M: Message,
//     {
//         let len = msg.encoded_len();
//         key_len(tag) + encoded_len_varint(len as u64) + len
//     }

//     #[inline]
//     pub fn encoded_len_repeated<M>(tag: u32, messages: &[M]) -> usize
//     where
//         M: Message,
//     {
//         key_len(tag) * messages.len()
//             + messages
//                 .iter()
//                 .map(Message::encoded_len)
//                 .map(|len| len + encoded_len_varint(len as u64))
//                 .sum::<usize>()
//     }
// }

// pub mod group {
//     use super::*;

//     pub fn encode<M, B>(tag: u32, msg: &M, buf: &mut B)
//     where
//         M: Message,
//         B: BufMut,
//     {
//         encode_key(tag, WireType::StartGroup, buf);
//         msg.encode_raw(buf);
//         encode_key(tag, WireType::EndGroup, buf);
//     }

//     pub fn encode_repeated<M, B>(tag: u32, messages: &[M], buf: &mut B)
//     where
//         M: Message,
//         B: BufMut,
//     {
//         for msg in messages {
//             encode(tag, msg, buf);
//         }
//     }

//     #[inline]
//     pub fn encoded_len<M>(tag: u32, msg: &M) -> usize
//     where
//         M: Message,
//     {
//         2 * key_len(tag) + msg.encoded_len()
//     }

//     #[inline]
//     pub fn encoded_len_repeated<M>(tag: u32, messages: &[M]) -> usize
//     where
//         M: Message,
//     {
//         2 * key_len(tag) * messages.len() + messages.iter().map(Message::encoded_len).sum::<usize>()
//     }
// }

/// Rust doesn't have a `Map` trait, so macros are currently the best way to be
/// generic over `HashMap` and `BTreeMap`.
macro_rules! map {
    ($map_ty:ident) => {
        use crate::encoding::*;
        use core::hash::Hash;

        /// Generic protobuf map encode function.
        pub fn encode<K, V, B, KE, KL, VE, VL>(
            key_encode: KE,
            key_encoded_len: KL,
            val_encode: VE,
            val_encoded_len: VL,
            tag: u32,
            values: &$map_ty<K, V>,
            buf: &mut B,
        ) where
            K: Default + Eq + Hash + Ord,
            V: Default + PartialEq,
            B: BufMut,
            KE: Fn(u32, &K, &mut B),
            KL: Fn(u32, &K) -> usize,
            VE: Fn(u32, &V, &mut B),
            VL: Fn(u32, &V) -> usize,
        {
            encode_with_default(
                key_encode,
                key_encoded_len,
                val_encode,
                val_encoded_len,
                &V::default(),
                tag,
                values,
                buf,
            )
        }

        /// Generic protobuf map encode function.
        pub fn encoded_len<K, V, KL, VL>(
            key_encoded_len: KL,
            val_encoded_len: VL,
            tag: u32,
            values: &$map_ty<K, V>,
        ) -> usize
        where
            K: Default + Eq + Hash + Ord,
            V: Default + PartialEq,
            KL: Fn(u32, &K) -> usize,
            VL: Fn(u32, &V) -> usize,
        {
            encoded_len_with_default(key_encoded_len, val_encoded_len, &V::default(), tag, values)
        }

        /// Generic protobuf map encode function with an overridden value default.
        ///
        /// This is necessary because enumeration values can have a default value other
        /// than 0 in proto2.
        pub fn encode_with_default<K, V, B, KE, KL, VE, VL>(
            key_encode: KE,
            key_encoded_len: KL,
            val_encode: VE,
            val_encoded_len: VL,
            val_default: &V,
            tag: u32,
            values: &$map_ty<K, V>,
            buf: &mut B,
        ) where
            K: Default + Eq + Hash + Ord,
            V: PartialEq,
            B: BufMut,
            KE: Fn(u32, &K, &mut B),
            KL: Fn(u32, &K) -> usize,
            VE: Fn(u32, &V, &mut B),
            VL: Fn(u32, &V) -> usize,
        {
            for (key, val) in values.iter() {
                let skip_key = key == &K::default();
                let skip_val = val == val_default;

                let len = (if skip_key { 0 } else { key_encoded_len(1, key) })
                    + (if skip_val { 0 } else { val_encoded_len(2, val) });

                encode_key(tag, WireType::LengthDelimited, buf);
                encode_varint(len as u64, buf);
                if !skip_key {
                    key_encode(1, key, buf);
                }
                if !skip_val {
                    val_encode(2, val, buf);
                }
            }
        }

        /// Generic protobuf map encode function with an overridden value default.
        ///
        /// This is necessary because enumeration values can have a default value other
        /// than 0 in proto2.
        pub fn encoded_len_with_default<K, V, KL, VL>(
            key_encoded_len: KL,
            val_encoded_len: VL,
            val_default: &V,
            tag: u32,
            values: &$map_ty<K, V>,
        ) -> usize
        where
            K: Default + Eq + Hash + Ord,
            V: PartialEq,
            KL: Fn(u32, &K) -> usize,
            VL: Fn(u32, &V) -> usize,
        {
            key_len(tag) * values.len()
                + values
                    .iter()
                    .map(|(key, val)| {
                        let len = (if key == &K::default() {
                            0
                        } else {
                            key_encoded_len(1, key)
                        }) + (if val == val_default {
                            0
                        } else {
                            val_encoded_len(2, val)
                        });
                        encoded_len_varint(len as u64) + len
                    })
                    .sum::<usize>()
        }
    };
}

pub mod hash_map {
    use std::collections::HashMap;
    map!(HashMap);
}

pub mod btree_map {
    map!(BTreeMap);
}
