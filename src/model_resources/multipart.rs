use std::sync::Arc;

use lasso::{Spur, ThreadedRodeo};
use serde_json::Value;

use super::{
    block_states::{InternedBlockState, State, StateName},
    variant::ModelVariant,
};

#[derive(Debug, Clone)]
pub struct Multipart {
    cases: Vec<Case>,
}

impl Multipart {
    pub fn get(
        &self,
        block_state: &InternedBlockState,
        rodeo: &Arc<ThreadedRodeo>,
    ) -> Vec<ModelVariant> {
        self.cases
            .iter()
            .filter_map(|case| {
                if case.check(block_state, rodeo) {
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
    pub fn check(&self, block_state: &InternedBlockState, rodeo: &Arc<ThreadedRodeo>) -> bool {
        self.when
            .as_ref()
            .is_none_or(|when| when.check(block_state, rodeo))
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
    pub fn check(&self, block_state: &InternedBlockState, rodeo: &Arc<ThreadedRodeo>) -> bool {
        match self {
            When::OrCase(test_block_states) => test_block_states.iter().any(|case_block_state| {
                case_block_state
                    .properties
                    .iter()
                    .all(|(case_state_name, case_state)| {
                        block_state.properties.iter().any(|(state_name, state)| {
                            state_name == case_state_name
                                && rodeo.resolve(case_state).split("|").any(|case_state_str| {
                                    rodeo
                                        .get(case_state_str)
                                        .is_some_and(|case_state_spur| case_state_spur == *state)
                                })
                        })
                    })
            }),
            When::AndCase(test_states) => test_states.iter().all(|case_block_state| {
                case_block_state
                    .properties
                    .iter()
                    .all(|(case_state_name, case_state)| {
                        block_state.properties.iter().any(|(state_name, state)| {
                            state_name == case_state_name && {
                                rodeo.resolve(case_state).split("|").any(|case_state_str| {
                                    rodeo
                                        .get(case_state_str)
                                        .is_some_and(|case_state_spur| case_state_spur == *state)
                                })
                            }
                        })
                    })
            }),
            When::SingleCase(test_block_state) => {
                test_block_state
                    .properties
                    .iter()
                    .all(|(case_state_name, case_state)| {
                        block_state.properties.iter().any(|(state_name, state)| {
                            state_name == case_state_name
                                && rodeo.resolve(case_state).split("|").any(|case_state_str| {
                                    rodeo
                                        .get(case_state_str)
                                        .is_some_and(|case_state_spur| case_state_spur == *state)
                                })
                        })
                    })
            }
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
        .map(|block_state_entry| {
            let properties: Vec<(Spur, Spur)> = block_state_entry
                .as_object()
                .expect("or case entry was not obj")
                .iter()
                .map(|(name, field)| {
                    let name_spur = rodeo.get_or_intern(name);
                    let field_spur = rodeo.get_or_intern(
                        field.as_str().expect("field in multipart case was not str"),
                    );
                    (name_spur, field_spur)
                })
                .collect();

            let r = InternedBlockState {
                properties: properties,
            };
            r
        })
        .collect::<Vec<_>>()
}
