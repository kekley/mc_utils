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

use super::resource_error::ResourceErrorKind;

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct BlockProperty<'a> {
    property_name: BumpString<'a>,
    property_value: BumpString<'a>,
}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct BlockProperties<'a> {
    pub names_values: BumpVec<'a, BlockProperty<'a>>,
}
/*                 let property_spur = property.split_once("=").map(|(state_name, state)| {
    let state_name = interner.get_or_intern(state_name);
    let state = interner.get_or_intern(state);
    (state_name, state)
}); */

impl<'a> BlockProperties<'a> {
    pub fn from_str(properties: &str, bump: &'a Bump) -> Result<Self, ResourceErrorKind> {
        let properties_split = properties.split(",");
        let result = properties_split
            .into_iter()
            .map(|property| {
                let (name, value) = property
                    .split_once("=")
                    .ok_or(ResourceErrorKind::InvalidField(format!("")))?;
                let name_string = BumpString::from_str_in(name, bump);
                let value_string = BumpString::from_str_in(value, bump);
                let property = BlockProperty {
                    property_name: name_string,
                    property_value: value_string,
                };
                Ok(property)
            })
            .collect_in::<Result<BumpVec<'a, _>, _>>(bump)?;

        Ok(BlockProperties {
            names_values: result,
        })
    }
}
