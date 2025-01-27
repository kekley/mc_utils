use bytes::{Buf, Bytes};
use core::str;
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use lasso::{Spur, ThreadedRodeo};
use num_enum::TryFromPrimitive;
use smol_str::SmolStr;
use std::{fmt::Debug, sync::Arc, u16};

use crate::spider_eye_error::SpiderEyeError;

use super::{
    nbt_ids::NBTId,
    nbt_tag::{get_nbt_string, NBTTag},
};

#[derive(Clone)]
pub struct NBTCompound {
    pub(crate) rodeo: Arc<ThreadedRodeo>,
    pub children: HashMap<Spur, NBTTag, FxBuildHasher>,
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
            res = res + &format!("Name: {}, ", self.rodeo.resolve(child.0));
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
    pub fn add_tag(&mut self, name: Spur, tag: NBTTag) {
        self.children.insert(name, tag);
    }
    pub fn get_tag(&self, tag_name: &str) -> Option<&NBTTag> {
        let spur = self.rodeo.get(tag_name)?;
        self.children.get(&spur)
    }

    pub fn from_bytes(
        stream: &mut Bytes,
        rodeo: Arc<ThreadedRodeo>,
    ) -> Result<Self, SpiderEyeError> {
        let mut tmp = Self {
            children: HashMap::with_hasher(FxBuildHasher::default()),
            rodeo: rodeo.clone(),
        };

        while stream.has_remaining() {
            let tag_id = NBTId::try_from_primitive(stream.get_u8())?;
            if tag_id == NBTId::EndId {
                break;
            }
            let name = get_nbt_string(stream, &rodeo)?;

            if let Ok(tag) = NBTTag::read_tag(stream, tag_id, rodeo.clone()) {
                tmp.add_tag(name, tag);
            } else {
                break;
            }
        }
        Ok(tmp)
    }
}
