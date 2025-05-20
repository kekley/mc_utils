use std::hash::Hash;

use lasso::{Interner, Spur};

use crate::MCResourceLoader;

pub type StateName = Spur;
pub type State = Spur;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InternedBlockState {
    pub properties: Vec<(StateName, State)>,
}
impl Hash for InternedBlockState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for prop in &self.properties {
            prop.hash(state);
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockState<'a> {
    pub properties: Vec<(&'a str, &'a str)>,
}

impl InternedBlockState {
    pub fn resolve<'a>(&'a self, loader: &'a MCResourceLoader) -> BlockState<'a> {
        let interner = &loader.rodeo;
        let properties_str: Vec<_> = self
            .properties
            .iter()
            .map(|(state_name, state)| (interner.resolve(state_name), interner.resolve(state)))
            .collect();
        BlockState {
            properties: properties_str,
        }
    }

    pub fn from_str(properties: &str, loader: &MCResourceLoader) -> Self {
        let interner = &loader.rodeo;
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
