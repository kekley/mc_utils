use bytes::{Buf, Bytes};
use core::str;
use fxhash::FxBuildHasher;
use num_enum::TryFromPrimitive;
use smol_str::SmolStr;
use std::{
    collections::{HashMap, HashSet},
    fmt::Debug,
    u16,
};

use crate::{
    compression::{self, CompressionData},
    nbt_ids::NBTId,
    nbt_tag::{get_nbt_string, NBTTag},
    spider_eye_error::SpiderEyeError,
};

#[derive(Clone)]
pub struct NBTCompound {
    pub seen_strings: HashSet<SmolStr, FxBuildHasher>,
    pub string_tags: Vec<SmolStr>,
    pub children: HashMap<SmolStr, NBTTag, FxBuildHasher>,
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
                    for _ in 0..indentation + 1 {
                        res = res + "\t"
                    }
                    let str = format!("{:?}", child.1);
                    res = res + &str;
                }
            }
        }
        res
    }
}

impl NBTCompound {
    pub fn add_tag(&mut self, name: SmolStr, tag: NBTTag) {
        self.children.insert(name, tag);
    }
    pub fn get_tag(&self, tag_name: &str) -> Option<&NBTTag> {
        self.children.get(tag_name)
    }

    pub fn from_bytes(stream: &mut Bytes) -> Result<Self, SpiderEyeError> {
        let mut tmp = Self {
            children: HashMap::with_hasher(FxBuildHasher::default()),
            seen_strings: HashSet::with_hasher(FxBuildHasher::default()),
            string_tags: vec![],
        };

        while stream.has_remaining() {
            let tag_id = NBTId::try_from_primitive(stream.get_u8())?;
            if tag_id == NBTId::EndId {
                break;
            }
            let name = get_nbt_string(stream)?;
            let name = SmolStr::from(str::from_utf8(&name).unwrap());

            if let Ok(tag) = NBTTag::read_tag(stream, tag_id) {
                tmp.add_tag(name, tag);
            } else {
                break;
            }
        }
        Ok(tmp)
    }
}
