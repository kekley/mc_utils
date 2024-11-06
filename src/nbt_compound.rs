use byteorder::ReadBytesExt;
use num_enum::TryFromPrimitive;
use std::{fmt::Debug, io::Read, u16};

use crate::{
    nbt_ids::NBTId,
    nbt_tag::{get_nbt_string, NBTTag, NamedTag},
    spider_eye_error::SpiderEyeError,
};

#[derive(Clone)]
pub struct NBTCompound {
    pub children: Vec<NamedTag>,
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
            res = res + &format!("Name: {}, ", child.name);
            match &child.tag {
                NBTTag::Compound(compound) => {
                    let str = compound.as_indented_string(indentation + 1);
                    res = res + &str;
                }
                _ => {
                    let str = format!("{:?}", child.tag);
                    res = res + &str;
                }
            }
        }
        res
    }
}

impl NBTCompound {
    pub fn add_tag(&mut self, name: String, tag: NBTTag) {
        self.children.push((name, tag).into());
    }
    pub fn from_borrowed_stream(stream: &mut dyn Read) -> Result<Self, SpiderEyeError> {
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
            let name = get_nbt_string(stream)?;
            if let Ok(tag) = NBTTag::read_tag(stream, tag_id) {
                tmp.add_tag(name, tag);
            } else {
                break;
            }
        }
        Ok(tmp)
    }
}
