use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use bytes::Buf;
use bytes::Bytes;
use cesu8::from_java_cesu8;
use core::str;
use num_enum::TryFromPrimitive;
use std::io::Read;
use std::{borrow::Cow, fmt::Debug, sync::Arc};

use super::java_string::JavaString;
use super::nbt_error::NBTError;
use super::nbt_error::NBTErrorKind;
use super::{nbt_compound::NBTCompound, nbt_ids::*};

#[repr(u8)]
#[derive(Clone, Debug)]
pub enum NBTTag<'a> {
    End = END_ID,
    Byte(i8) = BYTE_ID,
    Short(i16) = SHORT_ID,
    Int(i32) = INT_ID,
    Long(i64) = LONG_ID,
    Float(f32) = FLOAT_ID,
    Double(f64) = DOUBLE_ID,
    ByteArray(&'a [u8]) = BYTE_ARRAY_ID,
    String(JavaString<'a>) = STRING_ID,
    List(BumpVec<'a, NBTTag<'a>>) = LIST_ID,
    Compound(NBTCompound<'a>) = COMPOUND_ID,
    IntArray(&'a [u8]) = INT_ARRAY_ID,
    LongArray(&'a [u8]) = LONG_ARRAY_ID,
}

impl<'a> NBTTag<'a> {
    /// Returns the numeric id associated with the data type.
    pub const fn get_type_id(&self) -> u8 {
        // See https://doc.rust-lang.org/reference/items/enumerations.html#pointer-casting
        unsafe { *(self as *const Self as *const u8) }
    }
    pub fn read_tag<'b>(stream: &mut &'b [u8], bump: &'a Bump) -> Result<NBTTag<'a>, NBTError>
    where
        'b: 'a,
    {
        let (id, right) = stream.split_at(1);
        *stream = right;
        let id = NBTId::try_from_primitive(id[0])?;
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
                let (data, right) = stream.split_at(len);
                *stream = right;
                Ok(NBTTag::ByteArray(&data))
            }
            NBTId::StringId => Ok(NBTTag::String(JavaString::new(get_nbt_string_bytes(
                stream,
            ))?)),
            NBTId::ListId => {
                let tag_id = NBTId::try_from_primitive(stream.get_u8())?;
                let len = stream.get_i32() as usize;
                let mut list = BumpVec::with_capacity_in(len, bump);
                for _ in 0..len {
                    let tag = Self::read_tag(stream, bump)?;
                    if tag.get_type_id() != tag_id as u8 {
                        return Err(NBTError {
                            kind: NBTErrorKind::ListError(format!("")),
                        });
                    } else {
                        list.push(tag);
                    }
                }
                Ok(NBTTag::List(list))
            }
            NBTId::CompoundId => Ok(NBTTag::Compound(NBTCompound::new(stream, bump)?)),
            NBTId::IntArrayId => {
                let len = stream.get_i32() as usize;
                let (array, right) = stream.split_at(len * 4);

                Ok(NBTTag::IntArray(array))
            }
            NBTId::LongArrayId => {
                let len = stream.get_i32() as usize;
                let (array, right) = stream.split_at(len * 8);

                Ok(NBTTag::LongArray(array))
            }
        }
    }
}

impl<'a> NBTTag<'a> {
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
    pub fn get_byte_array(&self) -> &[u8] {
        if let NBTTag::ByteArray(value) = self {
            value
        } else {
            panic!("Tried to read a byte array from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_string(&self) -> Cow<'_, str> {
        if let NBTTag::String(value) = self {
            let str = value.to_str();
            str
        } else {
            panic!("Tried to read a string from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_list(&self) -> &[NBTTag] {
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
                    i32::from_be_bytes(s)
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
                    i64::from_be_bytes(s)
                })
                .collect();
            array
        } else {
            panic!("Tried to read a long array from a {:?}", self);
        }
    }
}

#[inline]
pub fn get_nbt_string_bytes<'a>(stream: &mut &'a [u8]) -> &'a [u8] {
    let len = stream.get_i16() as usize;
    let (left, right) = stream.split_at(len);
    *stream = right;
    left
}

// "takes" four bytes from the front of the slice, mutating it and produces an i32
pub fn take_from_slice<T>(slice: &mut &[u8]) -> T {
    let t_size = size_of::<T>();
    let value = &slice[0..t_size];
    let new_slice = &slice[t_size..];
    *slice = &new_slice;
    todo!()
}
