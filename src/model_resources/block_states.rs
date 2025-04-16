use std::sync::Arc;

use lasso::{Interner, Rodeo, Spur, ThreadedRodeo};

pub type StateName = Spur;
pub type State = Spur;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockStateInternal {
    pub(crate) properties: Vec<(StateName, State)>,
}

pub struct BlockState<'a> {
    pub properties: Vec<(&'a str, &'a str)>,
}

impl BlockStateInternal {
    pub fn resolve<'a>(&'a self, interner: &'a Arc<ThreadedRodeo>) -> BlockState<'a> {
        let properties_str: Vec<_> = self
            .properties
            .iter()
            .map(|(state_name, state)| (interner.resolve(state_name), interner.resolve(state)))
            .collect();
        BlockState {
            properties: properties_str,
        }
    }
    pub fn from_str(properties: &str, interner: &Arc<ThreadedRodeo>) -> Self {
        if properties.is_empty() {
            return Self { properties: vec![] };
        }
        let map = properties
            .split(",")
            .into_iter()
            .map(|property| {
                //                dbg!(property);
                let property = property
                    .split_once("=")
                    .map(|(state_name, state)| {
                        let state_name = interner.get_or_intern(state_name);
                        let state = interner.get_or_intern(state);
                        (state_name, state)
                    })
                    .expect("blockstate parse error");
                property
            })
            .collect::<Vec<(StateName, State)>>();

        Self { properties: map }
    }
}
