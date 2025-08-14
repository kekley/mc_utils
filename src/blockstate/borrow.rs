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

impl<'data, 'root_nbt> BlockState<'data, 'root_nbt> {
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

impl<'a, 'root_nbt> BlockStateTrait for BlockState<'a, 'root_nbt> {
    type StringType = &'a NBTStr;

    fn name(&self) -> Self::StringType {
        self.name
    }

    fn iter_properties(&self) -> impl Iterator<Item = (&Self::StringType, &Self::StringType)> {
        todo!()
    }
}
