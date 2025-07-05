use byteorder::ReadBytesExt;
use bytes::Buf;
use cesu8::from_java_cesu8;
use core::str;
use num_enum::TryFromPrimitive;
use std::borrow::Cow;
use std::fmt::Debug;
use std::io::{Cursor, Read};
use std::mem::ManuallyDrop;
use std::slice;

use super::nbt_error::{NBTError, NBTErrorKind};
use super::nbt_ids::*;

#[derive(Debug, Clone)]
pub struct NBTString {
    value: Vec<u8>,
}

impl NBTString {
    pub fn new(bytes: Vec<u8>) -> Result<Self, cesu8::Cesu8DecodingError> {
        if let Err(err) = from_java_cesu8(&bytes) {
            Err(err)
        } else {
            Ok(Self { value: bytes })
        }
    }
    pub fn as_str(&self) -> Cow<'_, str> {
        from_java_cesu8(&self.value)
            .expect("an NBTString should always contain a pre-checked string")
    }
}

#[derive(Clone)]
pub struct NBTCompound {
    children: Vec<(NBTString, NBTTag)>,
}
impl Debug for NBTCompound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compound:\n{:?}", self.children)
    }
}

impl NBTCompound {
    fn add_tag(&mut self, tag_name: NBTString, tag: NBTTag) {
        self.children.push((tag_name, tag));
    }
    pub fn get_tag(&self, tag_name: &str) -> Option<&NBTTag> {
        let tag = self.children.iter().find(|a| a.0.as_str() == tag_name);
        if let Some(child) = tag {
            return Some(&child.1);
        } else {
            return None;
        }
    }
    pub(crate) fn new<T: AsRef<[u8]>>(stream: &mut Cursor<T>) -> Result<Self, NBTError> {
        let mut tmp = NBTCompound { children: vec![] };

        while NBTId::try_from_primitive(stream.get_u8())? != NBTId::EndId {
            let name = get_nbt_string(stream)?;
            if let Ok(tag) = NBTTag::read_tag(stream) {
                tmp.add_tag(name, tag);
            } else {
                break;
            }
        }
        Ok(tmp)
    }
}

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
    ByteArray(Vec<u8>) = BYTE_ARRAY_ID,
    String(NBTString) = STRING_ID,
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
    pub fn read_tag<T: AsRef<[u8]>>(stream: &mut Cursor<T>) -> Result<NBTTag, NBTError> {
        let tag = stream.read_u8()?;
        let id = NBTId::try_from_primitive(tag)?;
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
                let mut buf = vec![0u8; len];
                stream.read_exact(&mut buf)?;
                Ok(NBTTag::ByteArray(buf))
            }
            NBTId::StringId => Ok(NBTTag::String(get_nbt_string(stream)?)),
            NBTId::ListId => {
                let tag_id = NBTId::try_from_primitive(stream.get_u8())?;
                let len = stream.get_i32() as usize;
                let mut list = Vec::with_capacity(len);
                for _ in 0..len {
                    let tag = Self::read_tag(stream)?;
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
            NBTId::CompoundId => Ok(NBTTag::Compound(NBTCompound::new(stream)?)),
            NBTId::IntArrayId => {
                let len = stream.get_i32() as usize;
                let vec = vec![0i32; len];
                let (ptr, len, cap) = {
                    let mut me = ManuallyDrop::new(vec);
                    (me.as_mut_ptr(), me.len(), me.capacity())
                };

                let buf: &mut [u8] = unsafe { slice::from_raw_parts_mut(ptr.cast(), len * 4) };

                stream.read_exact(buf)?;

                drop(buf);

                let mut vec_again = unsafe { Vec::from_raw_parts(ptr, len, cap) };

                vec_again
                    .iter_mut()
                    .for_each(|val| *val = i32::from_be(*val));

                Ok(NBTTag::IntArray(vec_again))
            }
            NBTId::LongArrayId => {
                //len * 8
                let len = stream.get_i32() as usize;

                let vec = vec![0i64; len];

                let (ptr, len, cap) = {
                    let mut me = ManuallyDrop::new(vec);
                    (me.as_mut_ptr(), me.len(), me.capacity())
                };

                let buf: &mut [u8] = unsafe { slice::from_raw_parts_mut(ptr.cast(), len * 8) };

                stream.read_exact(buf)?;

                drop(buf);

                let mut vec_again = unsafe { Vec::from_raw_parts(ptr, len, cap) };

                vec_again
                    .iter_mut()
                    .for_each(|val| *val = i64::from_be(*val));

                Ok(NBTTag::LongArray(vec_again))
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
    pub fn get_byte_array(&self) -> &[u8] {
        if let NBTTag::ByteArray(value) = self {
            value
        } else {
            panic!("Tried to read a byte array from a {:?}", self);
        }
    }
    #[inline]
    pub fn get_string(&self) -> &NBTString {
        if let NBTTag::String(value) = self {
            value
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

#[inline]
pub fn get_nbt_string<T: AsRef<[u8]>>(stream: &mut Cursor<T>) -> Result<NBTString, NBTError> {
    let len = stream.get_i16() as usize;
    let mut buf = vec![0u8; len];
    stream.read_exact(&mut buf)?;
    Ok(NBTString { value: buf })
}

// "takes" four bytes from the front of the slice, mutating it and produces an i32
pub fn take_from_slice<T>(slice: &mut &[u8]) -> T {
    let t_size = size_of::<T>();
    let value = &slice[0..t_size];
    let new_slice = &slice[t_size..];
    *slice = &new_slice;
    todo!()
}
