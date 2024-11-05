use std::{
    fmt::Debug,
    io::{Read, Write},
};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use num_enum::TryFromPrimitive;

use crate::{nbt_compound::NBTCompound, nbt_ids::*, region::RegionError};

#[repr(u8)]
#[derive(Clone)]
pub enum NBTTag {
    End = END_ID,
    Byte(i8) = BYTE_ID,
    Short(i16) = SHORT_ID,
    Int(i32) = INT_ID,
    Long(i64) = LONG_ID,
    Float(f32) = FLOAT_ID,
    Double(f64) = DOUBLE_ID,
    ByteArray(Bytes) = BYTE_ARRAY_ID,
    String(String) = STRING_ID,
    List(Vec<NBTTag>) = LIST_ID,
    Compound(NBTCompound) = COMPOUND_ID,
    IntArray(Vec<i32>) = INT_ARRAY_ID,
    LongArray(Vec<i64>) = LONG_ARRAY_ID,
}

impl NBTTag {
    /// Returns the numeric id associated with the data type.
    pub const fn get_type_id(&self) -> u8 {
        // See https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
        unsafe { *(self as *const Self as *const u8) }
    }
}

impl Debug for NBTTag {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::End => write!(f, "EndTag"),
            Self::Byte(arg0) => write!(f, "\nByte: {arg0}\n"),
            Self::Short(arg0) => write!(f, "\nShort: {arg0}\n"),
            Self::Int(arg0) => write!(f, "\nInt: {arg0}\n"),
            Self::Long(arg0) => write!(f, "\nLong: {arg0}\n"),
            Self::Float(arg0) => write!(f, "\nFloat: {arg0}\n"),
            Self::Double(arg0) => write!(f, "\nDouble: {arg0}\n"),
            Self::ByteArray(arg0) => f
                .debug_list()
                .entry(&"\nByte Array: ")
                .entries(arg0)
                .finish(),
            Self::String(arg0) => write!(f, "\n String: {arg0}\n"),
            Self::List(arg0) => write!(f, "\nList: {arg0:?}\n"),
            Self::Compound(arg0) => write!(f, "{:?}", arg0),
            Self::IntArray(arg0) => write!(f, "\nIntArray: {arg0:?}\n"),
            Self::LongArray(arg0) => write!(f, "\nIntArray: {arg0:?}\n"),
        }
    }
}
