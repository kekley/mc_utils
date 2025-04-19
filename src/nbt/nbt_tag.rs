use core::str;
use std::{borrow::Cow, fmt::Debug, mem::transmute_copy, ptr::slice_from_raw_parts, sync::Arc};

use anyhow::anyhow;
use bytes::{Buf, Bytes};
use cesu8::from_java_cesu8;
use lasso::ThreadedRodeo;
use num_enum::TryFromPrimitive;

use super::{nbt_compound::NBTCompound, nbt_ids::*};

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum NBTTag {
    End = END_ID,
    Byte(i8) = BYTE_ID,
    Short(i16) = SHORT_ID,
    Int(i32) = INT_ID,
    Long(i64) = LONG_ID,
    Float(f32) = FLOAT_ID,
    Double(f64) = DOUBLE_ID,
    ByteArray(Bytes) = BYTE_ARRAY_ID,
    String(Bytes) = STRING_ID,
    List(Vec<NBTTag>) = LIST_ID,
    Compound(NBTCompound) = COMPOUND_ID,
    IntArray(Bytes) = INT_ARRAY_ID,
    LongArray(Bytes) = LONG_ARRAY_ID,
}

impl NBTTag {
    /// Returns the numeric id associated with the data type.
    pub const fn get_type_id(&self) -> u8 {
        // See https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
        unsafe { *(self as *const Self as *const u8) }
    }
    pub fn read_tag(
        stream: &mut Bytes,
        id: NBTId,
        interner: &Arc<ThreadedRodeo>,
    ) -> anyhow::Result<NBTTag> {
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
                let data = stream.slice(0..len);
                stream.advance(len);
                Ok(NBTTag::ByteArray(data))
            }
            NBTId::StringId => Ok(NBTTag::String(get_nbt_string(stream)?)),
            NBTId::ListId => {
                let tag_id = NBTId::try_from_primitive(stream.get_u8())?;
                let len = stream.get_i32();
                let mut list = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let tag = Self::read_tag(stream, tag_id, interner)?;
                    if tag.get_type_id() != tag_id as u8 {
                        return Err(anyhow!(
                            "type of item in NBT list did not match declared list type"
                        ));
                    } else {
                        list.push(tag);
                    }
                }
                Ok(NBTTag::List(list))
            }
            NBTId::CompoundId => Ok(NBTTag::Compound(NBTCompound::internal_nbt(
                stream, interner,
            )?)),
            NBTId::IntArrayId => {
                let len = stream.get_i32() as usize;
                let bytes = stream.slice(0..len * 4);
                stream.advance(len * 4);

                Ok(NBTTag::IntArray(bytes))
            }
            NBTId::LongArrayId => {
                let len = stream.get_i32() as usize;
                let bytes = stream.slice(0..len * 8);
                stream.advance(len * 8);

                Ok(NBTTag::LongArray(bytes))
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
    pub fn get_byte_array(&self) -> &Bytes {
        if let NBTTag::ByteArray(value) = self {
            value
        } else {
            panic!("Tried to read a byte array from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_string(&self) -> Cow<'_, str> {
        if let NBTTag::String(value) = self {
            let str = from_java_cesu8(value).expect("invalid java string in the NBT file");
            str
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
    pub fn get_int_array(&self) -> Vec<i32> {
        if let NBTTag::IntArray(value) = self {
            let byte_slice = value.as_ref();
            let array = byte_slice
                .chunks_exact(4)
                .into_iter()
                .map(|s| {
                    assert!(s.len() == 4);
                    let s: [u8; 4] = s.try_into().unwrap();
                    i32::from_ne_bytes(s)
                })
                .collect();
            array
        } else {
            panic!("Tried to read an int array from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_long_array(&self) -> Vec<i64> {
        if let NBTTag::LongArray(value) = self {
            let byte_slice = value.as_ref();
            let array = byte_slice
                .chunks_exact(8)
                .into_iter()
                .map(|s| {
                    assert!(s.len() == 8);
                    let s: [u8; 8] = s.try_into().unwrap();
                    i64::from_ne_bytes(s)
                })
                .collect();
            array
        } else {
            panic!("Tried to read a long array from a {:?}", self);
        }
    }
}

#[inline]
pub fn get_nbt_string(stream: &mut Bytes) -> anyhow::Result<Bytes> {
    let len = stream.get_i16() as usize;
    let bytes = stream.slice(0..len);
    stream.advance(len);
    Ok(bytes)
}
