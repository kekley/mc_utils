#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use std::hash::Hash;

use super::block_states::BlockState;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct BlockName<'a> {
    value: BumpString<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block<'a> {
    pub block_name: BlockName<'a>,
    pub properties: BlockState<'a>,
}

impl<'a> Hash for Block<'a> {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.block_name.hash(state);
        self.properties.hash(state);
    }
}
