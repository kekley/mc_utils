use byteorder::{BigEndian, ReadBytesExt};
use bytes::{Buf, BufMut, Bytes, BytesMut};
use cesu8::from_java_cesu8;
use num_enum::TryFromPrimitive;
use std::io::{Error, Read};

use crate::{nbt_ids::NBTId, nbt_tag::NBTTag, region::RegionError};

#[derive(Debug)]
pub struct NBTCompound {
    pub children: Vec<(String, NBTTag)>,
}

impl NBTCompound {
    pub fn from_borrowed_stream(stream: &mut dyn Read) -> Result<Self, RegionError> {
        let tmp = Self { children: vec![] };
        let tag_id: NBTId = NBTId::try_from_primitive(stream.read_u8()?)?;

        let named_tags: Vec<(String, NBTTag)> = vec![];
    }

    pub fn read_list(stream: &mut dyn Read, id: NBTId) -> Result<Vec<NBTTag>, RegionError> {
        match id {
            NBTId::EndId => {
                let len = stream.read_i32::<BigEndian>()?;
                if len > 0 {
                    return Err(RegionError::ListError(len));
                }
                Ok(vec![NBTTag::End])
            }
            NBTId::ByteId => {
                let len = stream.read_i32::<BigEndian>()?;
                let mut data_list = Vec::with_capacity(len as usize);
                for _ in 0..len as usize {
                    let val = stream.read_i8()?;
                    data_list.push(NBTTag::Byte(val));
                }
                Ok(data_list)
            }
            NBTId::ShortId => {
                let len = stream.read_i32::<BigEndian>()?;
                let mut data_list = Vec::with_capacity(len as usize);
                for _ in 0..len as usize {
                    let val = stream.read_i16::<BigEndian>()?;
                    data_list.push(NBTTag::Short(val));
                }
                Ok(data_list)
            }
            NBTId::IntId => {
                let len = stream.read_i32::<BigEndian>()?;
                let mut data_list = Vec::with_capacity(len as usize);
                for _ in 0..len as usize {
                    let val = stream.read_i32::<BigEndian>()?;
                    data_list.push(NBTTag::Int(val));
                }
                Ok(data_list)
            }
            NBTId::LongId => {
                let len = stream.read_i32::<BigEndian>()?;
                let mut data_list = Vec::with_capacity(len as usize);
                for _ in 0..len as usize {
                    let val = stream.read_i64::<BigEndian>()?;
                    data_list.push(NBTTag::Long(val));
                }
                Ok(data_list)
            }
            NBTId::FloatId => {
                let len = stream.read_i32::<BigEndian>()?;
                let mut data_list = Vec::with_capacity(len as usize);
                for _ in 0..len as usize {
                    let val = stream.read_f32::<BigEndian>()?;
                    data_list.push(NBTTag::Float(val));
                }
                Ok(data_list)
            }
            NBTId::DoubleId => {
                let len = stream.read_i32::<BigEndian>()?;
                let mut data_list = Vec::with_capacity(len as usize);
                for _ in 0..len as usize {
                    let val = stream.read_f64::<BigEndian>()?;
                    data_list.push(NBTTag::Double(val));
                }
                Ok(data_list)
            }
            NBTId::ByteArrayId => {
                let len = stream.read_u8()?;
                let mut data_list = Vec::with_capacity(len as usize);
                for _ in 0..len as usize {
                    let val = Self::read_list(stream, NBTId::ByteArrayId)?;
                    data_list.push(NBTTag::List(val));
                }
                Ok(data_list)
            }
            NBTId::StringId => todo!(),
            NBTId::ListId => todo!(),
            NBTId::CompoundId => todo!(),
            NBTId::IntArrayId => todo!(),
            NBTId::LongArrayId => todo!(),
        }
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
