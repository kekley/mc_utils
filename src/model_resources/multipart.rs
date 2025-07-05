use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::{collections::CollectIn, Bump};
use serde_json::{Map, Value};

use super::utils::try_get_field;
use super::{
    block_states::BlockProperties, resource_error::ResourceErrorKind, utils::parse_type,
    variant::ModelVariant,
};

struct MultipartInner {}

#[derive(Debug, Clone)]
pub struct Multipart<'a> {
    cases: BumpVec<'a, Case<'a>>,
}

#[derive(Debug, Clone)]

pub struct Case<'a> {
    when: Option<When<'a>>,
    apply: Apply<'a>,
}
impl<'a> Case<'a> {
    pub fn check(&self, block_state: &BlockProperties) -> bool {
        if self
            .when
            .as_ref()
            .is_none_or(|when| when.check(block_state))
        {
            return true;
        } else {
            return false;
        }
    }
}
#[derive(Debug, Clone)]

pub struct Apply<'a> {
    variant: ModelVariant<'a>,
}

#[derive(Debug, Clone)]
pub struct TestStates<'a> {
    pub names_values: BumpVec<'a, (BumpString<'a>, BumpString<'a>)>,
}

#[derive(Debug, Clone)]
pub enum When<'a> {
    OrCase(BumpVec<'a, TestStates<'a>>),
    AndCase(BumpVec<'a, TestStates<'a>>),
    SingleCase(TestStates<'a>),
}
impl<'a> When<'a> {
    pub fn check(&self, block_state: &BlockProperties<'a>) -> bool {
        match self {
            When::OrCase(test_block_states) => {
                todo!()
            }
            When::AndCase(test_states) => {
                todo!()
            }
            When::SingleCase(test_block_state) => {
                todo!()
            }
        }
    }
}

impl<'a> Multipart<'a> {
    pub fn try_from_json(value: &Value, bump: &'a Bump) -> Result<Self, ResourceErrorKind> {
        let cases = parse_type::<Vec<Value>>(value)?
            .iter()
            .map(|value| Case::try_from_json(value, bump))
            .collect_in::<Result<BumpVec<'a, _>, ResourceErrorKind>>(bump)?;

        Ok(Multipart { cases: cases })
    }

    pub(crate) fn load_models(&self, properties: &BlockProperties<'_>) -> Vec<ModelVariant<'_>> {
        todo!()
    }
}

impl<'a> Case<'a> {
    pub fn try_from_json(value: &Value, bump: &'a Bump) -> Result<Self, ResourceErrorKind> {
        let when = value
            .get("when")
            .map(|value| {
                if let Some(value) = value.get("OR") {
                    //"Or" and "And" case are a list of test states, which are a json object containing an indeterminate number of fields in the format "state name" : "state_value(s)"
                    let test_states = Case::collect_test_states(value, bump)?;
                    Ok(When::OrCase(test_states))
                } else if let Some(value) = value.get("AND") {
                    let test_states = Case::collect_test_states(value, bump)?;
                    Ok(When::AndCase(test_states))
                } else {
                    let test_states = Case::collect_test_states(value, bump)?;
                    if test_states.len() > 0 {
                        return Err(ResourceErrorKind::InvalidField(
                            "Error parsing Case, empty single case".to_owned(),
                        ));
                    }
                    let test_state = test_states.into_iter().nth(0).take().unwrap();
                    Ok(When::SingleCase(test_state))
                }
            })
            .transpose()?;

        let variant = ModelVariant::from_json_value(try_get_field(value, "apply")?, bump)?;
        let apply = Apply { variant: variant };

        Ok(Case { when, apply: apply })
    }

    pub fn collect_test_states(
        value: &Value,
        bump: &'a Bump,
    ) -> Result<BumpVec<'a, TestStates<'a>>, ResourceErrorKind> {
        parse_type::<Vec<Value>>(value)?
            .iter()
            .map(|value| {
                let names_values = parse_type::<Map<String, Value>>(value)?
                    .iter()
                    .map(|(state_name, values)| {
                        let values_str = parse_type::<&str>(values)?;
                        Ok((
                            BumpString::from_str_in(&state_name, bump),
                            BumpString::from_str_in(values_str, bump),
                        ))
                    })
                    .collect_in::<Result<BumpVec<_>, ResourceErrorKind>>(bump)?; // if we fail parsing at any point we want to just return an error for the whole thing
                Ok(TestStates { names_values })
            })
            .collect_in::<Result<BumpVec<'a, _>, ResourceErrorKind>>(bump)
    }
}
