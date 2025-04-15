use bytes::{Buf, Bytes};
use cesu8::from_java_cesu8;
use core::str;
use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use lasso::{Rodeo, Spur};
use num_enum::TryFromPrimitive;
use std::fmt::Debug;

use super::{
    nbt_ids::NBTId,
    nbt_tag::{get_nbt_string, NBTTag},
};

#[derive(Clone)]
pub struct NBTCompound {
    string_interner: Rodeo,
    pub children: HashMap<Spur, NBTTag, FxBuildHasher>,
}
impl Debug for NBTCompound {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Compound:\n{:?}", self.children)
    }
}

impl NBTCompound {
    pub fn add_tag(&mut self, tag_name: &str, tag: NBTTag) {
        let spur = self.string_interner.get_or_intern(tag_name);
        self.children.insert(spur, tag);
    }
    pub fn get_tag(&self, tag_name: &str) -> Option<&NBTTag> {
        let spur = self.string_interner.get(tag_name)?;
        self.children.get(&spur)
    }

    pub(crate) fn from_bytes(stream: &mut Bytes) -> anyhow::Result<Self> {
        let interner = Rodeo::new();
        let mut tmp = Self {
            string_interner: interner,
            children: HashMap::with_hasher(FxBuildHasher::default()),
        };

        while stream.has_remaining() {
            let tag_id = NBTId::try_from_primitive(stream.get_u8())?;
            if tag_id == NBTId::EndId {
                break;
            }
            let string_bytes = get_nbt_string(stream)?;
            let name = from_java_cesu8(&string_bytes)?;
            if let Ok(tag) = NBTTag::read_tag(stream, tag_id) {
                tmp.add_tag(&name, tag);
            } else {
                break;
            }
        }
        Ok(tmp)
    }
}
