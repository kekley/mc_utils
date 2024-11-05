use std::io::{Read, Write};

use byteorder::{BigEndian, ReadBytesExt};
use bytes::Bytes;
use num_enum::TryFromPrimitive;

use crate::{nbt_compound::NBTCompound, nbt_ids::*, region::RegionError};

#[repr(u8)]
#[derive(Debug)]
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

pub fn parse_nbt_bytes(stream: &mut dyn Read) -> Result<(), RegionError> {
    let tmp = NBTCompound { children: vec![] };

    Ok(())
}
