use std::{slice, sync::Arc};

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
        println!("{:?}", self.cases.len());
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
        if self
            .when
            .as_ref()
            .is_none_or(|when| when.check(block_state, rodeo))
        {
            return true;
        } else {
            return false;
        }
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
            When::OrCase(test_block_states) => {
                test_block_states
                    .iter()
                    .enumerate()
                    .any(|(index, case_block_state)| {
                        case_block_state
                            .properties
                            .iter()
                            .all(|(case_state_name, case_state)| {
                                block_state.properties.iter().any(|(state_name, state)| {
                                    state_name == case_state_name
                                        && rodeo.resolve(case_state).split("|").any(
                                            |case_state_str| {
                                                let val = rodeo.get(case_state_str).is_some_and(
                                                    |case_state_spur| case_state_spur == *state,
                                                );
                                                dbg!("And Case chose state index", index);
                                                val
                                            },
                                        )
                                })
                            })
                    })
            }
            When::AndCase(test_states) => {
                test_states
                    .iter()
                    .enumerate()
                    .all(|(index, case_block_state)| {
                        case_block_state
                            .properties
                            .iter()
                            .all(|(case_state_name, case_state)| {
                                block_state.properties.iter().any(|(state_name, state)| {
                                    state_name == case_state_name && {
                                        rodeo.resolve(case_state).split("|").any(|case_state_str| {
                                            let val = rodeo.get(case_state_str).is_some_and(
                                                |case_state_spur| case_state_spur == *state,
                                            );
                                            dbg!("And Case chose state index", index);
                                            val
                                        })
                                    }
                                })
                            })
                    })
            }
            When::SingleCase(test_block_state) => {
                let a = test_block_state
                    .properties
                    .iter()
                    .all(|(test_state_name, test_state)| {
                        block_state
                            .properties
                            .iter()
                            .any(|(tested_state_name, tested_state)| {
                                if tested_state_name == test_state_name {
                                    let mut split = rodeo.resolve(test_state).split("|");
                                    split.any(|case_state_str| {
                                        let val = rodeo.get(case_state_str).is_some_and(
                                            |case_state_spur| case_state_spur == *tested_state,
                                        );

                                        val
                                    })
                                } else {
                                    false
                                }
                            })
                    });
                if a {
                    println!("Chose single case");
                    println!(
                        "tested: {:?}, tested against:{:?}",
                        block_state.resolve(rodeo),
                        test_block_state.resolve(rodeo)
                    );
                }
                a
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
                let a = collect_blockstates(slice::from_ref(&value), rodeo);
                let a = a[0].clone();
                let block_state = a;
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

fn collect_blockstates(value: &[Value], rodeo: &Arc<ThreadedRodeo>) -> Vec<InternedBlockState> {
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
