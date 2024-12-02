use std::{any::Any, fmt::Debug, ops::Deref};

use bytes::{buf, Buf, Bytes};
use cesu8::from_java_cesu8;
use num_enum::TryFromPrimitive;
use smol_str::SmolStr;

use crate::{nbt_compound::NBTCompound, nbt_ids::*, spider_eye_error::SpiderEyeError};

#[derive(Debug, Clone)]
pub struct NamedTag {
    pub name: SmolStr,
    pub tag: NBTTag,
}

impl From<(SmolStr, NBTTag)> for NamedTag {
    fn from(value: (SmolStr, NBTTag)) -> Self {
        Self {
            name: value.0,
            tag: value.1,
        }
    }
}

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
    ByteArray(Vec<i8>) = BYTE_ARRAY_ID,
    String(SmolStr) = STRING_ID,
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
    pub fn read_tag(stream: &mut dyn Buf, id: NBTId) -> Result<NBTTag, SpiderEyeError> {
        match id {
            NBTId::EndId => Ok(NBTTag::End),
            NBTId::ByteId => Ok(NBTTag::Byte(stream.get_i8())),
            NBTId::ShortId => Ok(NBTTag::Short(stream.get_i16())),
            NBTId::IntId => Ok(NBTTag::Int(stream.get_i32())),
            NBTId::LongId => Ok(NBTTag::Long(stream.get_i64())),
            NBTId::FloatId => Ok(NBTTag::Float(stream.get_f32())),
            NBTId::DoubleId => Ok(NBTTag::Double(stream.get_f64())),
            NBTId::ByteArrayId => {
                let len = stream.get_i32() as usize;
                let mut data = Vec::with_capacity(len);
                for _ in 0..len {
                    data.push(stream.get_i8());
                }
                Ok(NBTTag::ByteArray(data))
            }
            NBTId::StringId => Ok(NBTTag::String(get_nbt_string(stream)?)),
            NBTId::ListId => {
                let tag_id = NBTId::try_from_primitive(stream.get_u8())?;
                let len = stream.get_i32();
                let mut list = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let tag = Self::read_tag(stream, tag_id)?;
                    if tag.get_type_id() != tag_id as u8 {
                        return Err(SpiderEyeError::ListError(len));
                    } else {
                        list.push(tag);
                    }
                }
                Ok(NBTTag::List(list))
            }
            NBTId::CompoundId => Ok(NBTTag::Compound(NBTCompound::from_borrowed_stream(stream)?)),
            NBTId::IntArrayId => {
                let len = stream.get_i32() as usize;
                let mut array = Vec::with_capacity(len);
                for _ in 0..len {
                    array.push(stream.get_i32());
                }
                Ok(NBTTag::IntArray(array))
            }
            NBTId::LongArrayId => {
                let len = stream.get_i32() as usize;
                let mut array = Vec::with_capacity(len);
                for _ in 0..len {
                    array.push(stream.get_i64());
                }
                Ok(NBTTag::LongArray(array))
            }
        }
    }
}

impl NBTTag {
    #[inline]
    pub fn get_byte(&self) -> i8 {
        if let NBTTag::Byte(value) = self {
            *value
        } else {
            panic!("Tried to read a byte from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_short(&self) -> i16 {
        if let NBTTag::Short(value) = self {
            *value
        } else {
            panic!("Tried to read a short from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_int(&self) -> i32 {
        if let NBTTag::Int(value) = self {
            *value
        } else {
            panic!("Tried to read an int from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_long(&self) -> i64 {
        if let NBTTag::Long(value) = self {
            *value
        } else {
            panic!("Tried to read a long from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_float(&self) -> f32 {
        if let NBTTag::Float(value) = self {
            *value
        } else {
            panic!("Tried to read a float from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_double(&self) -> f64 {
        if let NBTTag::Double(value) = self {
            *value
        } else {
            panic!("Tried to read a double from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_byte_array(&self) -> &Vec<i8> {
        if let NBTTag::ByteArray(value) = self {
            value
        } else {
            panic!("Tried to read a byte array from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_string(&self) -> &SmolStr {
        if let NBTTag::String(value) = self {
            value
        } else {
            panic!("Tried to read a string from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_list(&self) -> &Vec<NBTTag> {
        if let NBTTag::List(value) = self {
            value
        } else {
            panic!("Tried to read a list from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_compound(&self) -> &NBTCompound {
        if let NBTTag::Compound(value) = self {
            value
        } else {
            panic!("Tried to read a compound from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_int_array(&self) -> &Vec<i32> {
        if let NBTTag::IntArray(value) = self {
            value
        } else {
            panic!("Tried to read an int array from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_long_array(&self) -> &Vec<i64> {
        if let NBTTag::LongArray(value) = self {
            value
        } else {
            panic!("Tried to read a long array from a {:?}", self);
        }
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
pub fn get_nbt_string(stream: &mut dyn Buf) -> Result<SmolStr, SpiderEyeError> {
    let len = stream.get_i16() as usize;
    let a = stream.copy_to_bytes(len);

    let string = from_java_cesu8(&a)?;
    Ok(SmolStr::from(string))
}
