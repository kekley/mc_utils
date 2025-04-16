use std::sync::Arc;

use lasso::{Spur, ThreadedRodeo};
use serde_json::Value;

use super::{
    block_states::{BlockState, InternedBlockState},
    variant::ModelVariant,
};

#[derive(Debug, Clone)]
pub struct Multipart {
    cases: Vec<Case>,
}

impl Multipart {
    pub fn get(&self, block_state: &InternedBlockState) -> Vec<ModelVariant> {
        self.cases
            .iter()
            .filter_map(|case| {
                if case.check(block_state) {
                    Some(case.apply.variant.clone())
                } else {
                    None
                }
            })
            .collect()
    }
}
#[derive(Debug, Clone)]

pub struct Case {
    when: Option<When>,
    apply: Apply,
}
impl Case {
    pub fn check(&self, block_state: &InternedBlockState) -> bool {
        self.when
            .as_ref()
            .is_none_or(|when| when.check(block_state))
    }
}
#[derive(Debug, Clone)]

pub struct Apply {
    variant: ModelVariant,
}

#[derive(Debug, Clone)]
pub enum When {
    OrCase(Vec<InternedBlockState>),
    AndCase(Vec<InternedBlockState>),
    SingleCase(InternedBlockState),
}
impl When {
    pub fn check(&self, block_state: &InternedBlockState) -> bool {
        match self {
            When::OrCase(test_states) => test_states.iter().any(|state| state == block_state),
            When::AndCase(test_states) => test_states.iter().all(|state| state == block_state),
            When::SingleCase(test_state) => test_state == block_state,
        }
    }
}

impl Multipart {
    pub fn new(value: &Value, rodeo: &Arc<ThreadedRodeo>) -> Self {
        let cases = value
            .as_array()
            .expect("multipart was not array of cases")
            .iter()
            .map(|value| Case::new(value, &rodeo))
            .collect::<Vec<_>>();
        Multipart { cases: cases }
    }
}

impl Case {
    pub fn new(value: &Value, rodeo: &Arc<ThreadedRodeo>) -> Self {
        let when = value.get("when").map(|value| {
            if let Some(value) = value.get("OR") {
                let array = value.as_array().expect("OR case was not array");
                let block_states = collect_blockstates(array, rodeo);
                When::OrCase(block_states)
            } else if let Some(value) = value.get("AND") {
                let array = value.as_array().expect("AND case was not array");
                let block_states = collect_blockstates(array, rodeo);
                When::AndCase(block_states)
            } else {
                let (name, state) = value
                    .as_object()
                    .unwrap()
                    .iter()
                    .map(|f| {
                        let state_name = rodeo.get_or_intern(f.0);
                        let state = rodeo
                            .get_or_intern(f.1.as_str().expect("single when case was not str"));
                        (state_name, state)
                    })
                    .collect::<Vec<_>>()
                    .first()
                    .cloned()
                    .unwrap();
                let map: Vec<(Spur, Spur)> = vec![(name, state)];

                let block_state = InternedBlockState { properties: map };
                When::SingleCase(block_state)
            }
        });

        let variant = ModelVariant::from_json_value(
            value
                .get("apply")
                .expect("no model specified for multipart"),
            rodeo,
        );
        let apply = Apply { variant: variant };

        Case { when, apply: apply }
    }
}

fn collect_blockstates(value: &Vec<Value>, rodeo: &Arc<ThreadedRodeo>) -> Vec<InternedBlockState> {
    value
        .iter()
        .flat_map(|entry| {
            entry
                .as_object()
                .expect("or case entry was not obj")
                .iter()
                .map(|obj| {
                    let state_name = rodeo.get_or_intern(obj.0);
                    let values = obj.1.as_str().expect("values were not str").split("|");
                    let map = values
                        .into_iter()
                        .map(|value| (state_name, rodeo.get_or_intern(value)))
                        .collect();
                    InternedBlockState { properties: map }
                })
        })
        .collect::<Vec<InternedBlockState>>()
}
