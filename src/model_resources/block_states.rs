#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::{collections::CollectIn, Bump};
use std::hash::Hash;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct BlockState<'a> {
    pub properties: BumpString<'a>,
}
/*                 let property_spur = property.split_once("=").map(|(state_name, state)| {
    let state_name = interner.get_or_intern(state_name);
    let state = interner.get_or_intern(state);
    (state_name, state)
}); */

impl<'a> BlockState<'a> {
    pub fn from_str(properties: &str, bump: &'a Bump) -> Self {
        BlockState {
            properties: BumpString::from_str_in(properties, bump),
        }
    }
}
