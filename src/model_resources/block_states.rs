use std::hash::Hash;

use lasso::{Interner, Spur};

use crate::MCResourceLoader;

pub type StateName = Spur;
pub type State = Spur;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InternedBlockState {
    //vec of properties, where properties are arranged as: (property_name=value)
    pub properties: Vec<(StateName, State)>,
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockState<'a> {
    pub properties: Vec<(&'a str, &'a str)>,
}

impl InternedBlockState {
    pub fn from_str(properties: &str, loader: &MCResourceLoader) -> Self {
        if properties.is_empty() {
            return Self { properties: vec![] };
        }
        let map = properties
            .split(",")
            .into_iter()
            .filter_map(|property| {
                //                dbg!(property);
                let property_spur = property.split_once("=").map(|(state_name, state)| {
                    let state_name = interner.get_or_intern(state_name);
                    let state = interner.get_or_intern(state);
                    (state_name, state)
                });
                if property_spur.is_none() {
                    //TODO
                }
                property_spur
            })
            .collect::<Vec<(StateName, State)>>();

        let r = Self { properties: map };
        r
    }
}
