use std::hash::Hash;

use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use lasso::{Spur, ThreadedRodeo};

pub type StateName = Spur;
pub type State = Spur;
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct InternalBlockState {
    pub(crate) properties: HashMap<StateName, State, FxBuildHasher>,
}

impl Hash for InternalBlockState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        for (key, value) in &self.properties {
            key.hash(state);
            value.hash(state);
        }
    }
}

impl InternalBlockState {
    pub fn from_str(properties: &str, rodeo: &ThreadedRodeo) -> Self {
        if properties.is_empty() {
            return Self {
                properties: HashMap::with_hasher(FxBuildHasher::default()),
            };
        }
        let map = properties
            .split(",")
            .into_iter()
            .map(|property| {
                //                dbg!(property);
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

        Self { properties: map }
    }
}
