use std::fmt::{Debug, Display};

use crate::borrow::{
    nbt_compound::{NBTCompound, NBTCompoundIter},
    nbt_string::NBTStr,
};

use super::BlockStateTrait;

#[derive(Clone)]
pub struct BlockState<'data, 'root_nbt> {
    name: &'data NBTStr,
    properties: Option<NBTCompound<'data, 'root_nbt>>,
}

#[expect(unsafe_code)]
const AIR_NAME: &NBTStr = const {
    //SAFETY: "minecraft:air" is valid mutf8
    unsafe { NBTStr::from_str_unchecked("minecraft:air") }
};

impl<'data, 'root_nbt> BlockState<'data, 'root_nbt> {
    const AIR: BlockState<'static, 'static> = BlockState {
        name: AIR_NAME,
        properties: None,
    };

    pub fn get_name(&self) -> &'data NBTStr {
        self.name
    }
    pub fn iter_properties(&self) -> PropertiesIter<'data, 'root_nbt> {
        let iter = self.properties.as_ref().map(|compound| compound.iter());
        PropertiesIter { iter }
    }
    pub fn from_compound(compound: NBTCompound<'data, 'root_nbt>) -> Option<Self> {
        let name = compound.get_tag("Name")?;
        let name = name.get_string()?;

        let properties = if let Some(properties) = compound.get_tag("Properties") {
            properties.get_compound()
        } else {
            None
        };

        Some(Self { name, properties })
    }
}

impl Debug for BlockState<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_fmt(format_args!("{}", self))
    }
}

impl Display for BlockState<'_, '_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Block name: ")?;
        f.write_str(&self.get_name().to_str())?;
        f.write_str("\n")?;

        f.write_str("Properties:\n")?;

        for (name, val) in self.iter_properties() {
            f.write_str(&name.to_str())?;
            f.write_str(": ")?;
            f.write_str(&val.to_str())?;
        }

        Ok(())
    }
}

pub struct PropertiesIter<'data, 'root_nbt> {
    iter: Option<NBTCompoundIter<'data, 'root_nbt>>,
}

impl<'a, 'root_nbt> Iterator for PropertiesIter<'a, 'root_nbt> {
    type Item = (&'a NBTStr, &'a NBTStr);

    fn next(&mut self) -> Option<Self::Item> {
        let tmp = self.iter.as_mut()?;
        for (name, tag) in tmp.by_ref() {
            if let Some(value) = tag.get_string() {
                return Some((name, value));
            }
        }
        None
    }
}

impl<'a, 'root_nbt> BlockStateTrait<'a> for BlockState<'a, 'root_nbt> {
    type StringType = &'a NBTStr;

    fn name(&self) -> Self::StringType {
        self.name
    }

    fn iter_properties(&self) -> PropertiesIter<'a, 'root_nbt> {
        todo!();
    }
}
