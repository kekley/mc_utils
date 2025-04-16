use bytes::{Buf, Bytes};
use cesu8::from_java_cesu8;
use core::str;
use lasso::{Interner, Reader, Spur, ThreadedRodeo};
use num_enum::TryFromPrimitive;
use std::{fmt::Debug, sync::Arc};

use crate::palette::InternerType;

use super::{
    nbt_ids::NBTId,
    nbt_tag::{get_nbt_string, NBTTag},
};

pub type NBTTagName = Spur;

#[derive(Clone)]
pub struct NBTCompound {
    string_interner: InternerType,
    pub children: Vec<(NBTTagName, NBTTag)>,
}
impl Debug for NBTCompound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compound:\n{:?}", self.children)
    }
}

impl NBTCompound {
    pub fn add_tag(&mut self, tag_name: &str, tag: NBTTag) {
        let spur = self.string_interner.get_or_intern(tag_name);
        self.children.push((spur, tag));
    }
    pub fn get_tag(&self, tag_name: &str) -> Option<&NBTTag> {
        let spur = self.string_interner.get(tag_name)?;
        let tag = self.children.iter().find(|child| child.0 == spur);
        if let Some(child) = tag {
            return Some(&child.1);
        } else {
            return None;
        }
    }

    pub fn from_bytes(stream: &mut Bytes) -> anyhow::Result<Self> {
        let interner = Arc::new(ThreadedRodeo::new());
        let mut tmp = Self {
            string_interner: InternerType::External(interner.clone()),
            children: vec![],
        };

        while stream.has_remaining() {
            let tag_id = NBTId::try_from_primitive(stream.get_u8())?;
            if tag_id == NBTId::EndId {
                break;
            }
            let string_bytes = get_nbt_string(stream)?;
            let name = from_java_cesu8(&string_bytes)?;
            if let Ok(tag) = NBTTag::read_tag(stream, tag_id, &interner) {
                tmp.add_tag(&name, tag);
            } else {
                break;
            }
        }
        Ok(tmp)
    }

    pub(crate) fn internal_nbt(
        stream: &mut Bytes,
        interner: &Arc<ThreadedRodeo>,
    ) -> anyhow::Result<Self> {
        let mut tmp = Self {
            string_interner: InternerType::External(interner.clone()),
            children: vec![],
        };

        while stream.has_remaining() {
            let tag_id = NBTId::try_from_primitive(stream.get_u8())?;
            if tag_id == NBTId::EndId {
                break;
            }
            let string_bytes = get_nbt_string(stream)?;
            let name = from_java_cesu8(&string_bytes)?;
            if let Ok(tag) = NBTTag::read_tag(stream, tag_id, interner) {
                tmp.add_tag(&name, tag);
            } else {
                break;
            }
        }
        Ok(tmp)
    }
}
