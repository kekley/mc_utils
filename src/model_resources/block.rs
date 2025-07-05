use std::hash::Hash;

use crate::owned::nbt_string::NBTString;

use super::block_states::BlockProperties;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct BlockName {
    value: NBTString,
}

impl BlockName {
    pub fn new_from_str(name: &str) -> Self {
        Self {
            value: NBTString::new_from_str(name).unwrap(),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block {
    pub block_name: BlockName,
    pub properties: BlockProperties<'static>,
}

impl<'a> Hash for Block {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.block_name.hash(state);
        self.properties.hash(state);
    }
}
