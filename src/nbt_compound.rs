use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use cesu8::from_java_cesu8;
use fastnbt::stream::Name;
use num_enum::TryFromPrimitive;
use std::{
    fmt::Debug,
    io::{Error, Read},
    u16,
};

use crate::{nbt_ids::NBTId, nbt_tag::NBTTag, region::RegionError};

#[derive(Clone)]
pub struct NBTCompound {
    pub children: Vec<(String, NBTTag)>,
}
impl Debug for NBTCompound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compound:\n{:?}", self.children)
    }
}

impl NBTCompound {
    pub fn as_indented_string(&self, indentation: u16) -> String {
        let mut res = String::with_capacity(1024 * 38);
        for child in &self.children {
            for _ in 0..indentation + 1 {
                res = res + "\t"
            }
            res = res + &format!("Name: {}, ", child.0);
            match &child.1 {
                NBTTag::Compound(compound) => {
                    let str = compound.as_indented_string(indentation + 1);
                    res = res + &str;
                }
                _ => {
                    let str = format!("{:?}", child.1);
                    res = res + &str;
                }
            }
        }
        res
    }
}

impl NBTCompound {
    pub fn add_tag(&mut self, name: String, tag: NBTTag) {
        self.children.push((name, tag));
    }
    pub fn from_borrowed_stream(stream: &mut dyn Read) -> Result<Self, RegionError> {
        let mut tmp = Self { children: vec![] };

        loop {
            let result = stream.read_u8();
            let tag_id;
            match result {
                Ok(val) => {
                    tag_id = val;
                }
                Err(e) => {
                    println!("EOF");
                    break;
                }
            }
            let tag_id = NBTId::try_from_primitive(tag_id)?;
            if tag_id == NBTId::EndId {
                break;
            }
            let name = Self::get_nbt_string(stream)?;
            if let Ok(tag) = Self::read_tag(stream, tag_id) {
                tmp.add_tag(name, tag);
            } else {
                break;
            }
        }
        Ok(tmp)
    }

    pub fn read_tag(stream: &mut dyn Read, id: NBTId) -> Result<NBTTag, RegionError> {
        match id {
            NBTId::EndId => Ok(NBTTag::End),
            NBTId::ByteId => Ok(NBTTag::Byte(stream.read_i8()?)),
            NBTId::ShortId => Ok(NBTTag::Short(stream.read_i16::<BigEndian>()?)),
            NBTId::IntId => Ok(NBTTag::Int(stream.read_i32::<BigEndian>()?)),
            NBTId::LongId => Ok(NBTTag::Long(stream.read_i64::<BigEndian>()?)),
            NBTId::FloatId => Ok(NBTTag::Float(stream.read_f32::<BigEndian>()?)),
            NBTId::DoubleId => Ok(NBTTag::Double(stream.read_f64::<BigEndian>()?)),
            NBTId::ByteArrayId => {
                let len = stream.read_i32::<BigEndian>()?;
                let mut data = vec![0u8; len as usize];
                stream.read_exact(&mut data[..])?;
                Ok(NBTTag::ByteArray(data.into()))
            }
            NBTId::StringId => Ok(NBTTag::String(Self::get_nbt_string(stream)?)),
            NBTId::ListId => {
                let tag_id = NBTId::try_from_primitive(stream.read_u8()?)?;
                let len = stream.read_i32::<BigEndian>()?;
                let mut list = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    let tag = Self::read_tag(stream, tag_id)?;
                    if tag.get_type_id() != tag_id as u8 {
                        return Err(RegionError::ListError(len));
                    } else {
                        list.push(tag);
                    }
                }
                Ok(NBTTag::List(list))
            }
            NBTId::CompoundId => Ok(NBTTag::Compound(NBTCompound::from_borrowed_stream(stream)?)),
            NBTId::IntArrayId => {
                let len = stream.read_i32::<BigEndian>()?;
                let mut array = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    array.push(stream.read_i32::<BigEndian>()?);
                }
                Ok(NBTTag::IntArray(array))
            }
            NBTId::LongArrayId => {
                let len = stream.read_i32::<BigEndian>()?;
                let mut array = Vec::with_capacity(len as usize);
                for _ in 0..len {
                    array.push(stream.read_i64::<BigEndian>()?);
                }
                Ok(NBTTag::LongArray(array))
            }
        }
    }

    pub fn get_nbt_string(stream: &mut dyn Read) -> Result<String, RegionError> {
        let len = stream.read_u16::<BigEndian>()? as usize;
        let mut string_bytes = vec![0u8; len];
        stream.read_exact(&mut string_bytes[..])?;
        let string = from_java_cesu8(&string_bytes[..])?;
        Ok(string.to_string())
    }
}
