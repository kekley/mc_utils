use std::{hash::Hash, sync::Arc};

use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use lasso::{Spur, ThreadedRodeo};
use serde_json::Value;
use smol_str::SmolStr;

pub type StateName = Spur;
pub type State = Spur;
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockState {
    pub(crate) properties: Option<HashMap<StateName, State, FxBuildHasher>>,
}

impl Hash for BlockState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        if let Some(properties) = &self.properties {
            for (key, value) in properties {
                key.hash(state);
                value.hash(state);
            }
        }
    }
}

impl BlockState {
    pub fn new(properties: &str, rodeo: &ThreadedRodeo) -> Self {
        if properties.is_empty() {
            return Self { properties: None };
        }
        let split = properties
            .split(",")
            .into_iter()
            .map(|property| {
                let property = property
                    .split_once("=")
                    .map(|(state_name, state)| {
                        let state_name = rodeo.get_or_intern(state_name);
                        let state = rodeo.get_or_intern(state);
                        (state_name, state)
                    })
                    .expect("blockstate parse error");
                property
            })
            .collect::<HashMap<_, _, _>>();

        Self {
            properties: Some(split),
        }
    }
}
