#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use std::hash::Hash;

use super::block_states::BlockProperties;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct BlockName<'a> {
    value: BumpString<'a>,
}

impl<'a> BlockName<'a> {
    pub fn new_in(name: &str, bump: &'a Bump) -> Self {
        Self {
            value: BumpString::from_str_in(name, bump),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block<'a> {
    pub block_name: BlockName<'a>,
    pub properties: BlockProperties<'a>,
}

impl<'a> Hash for Block<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.block_name.hash(state);
        self.properties.hash(state);
    }
}
