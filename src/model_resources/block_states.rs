use std::sync::Arc;

use lasso::{Interner, Rodeo, Spur, ThreadedRodeo};

pub type StateName = Spur;
pub type State = Spur;
#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockStateInternal {
    pub(crate) properties: Vec<(StateName, State)>,
}

impl BlockStateInternal {
    pub fn from_str(properties: &str, interner: &mut dyn Interner) -> Self {
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
