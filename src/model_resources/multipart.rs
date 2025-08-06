use serde_json::{Map, Value};

use super::utils::try_get_field;
use super::{resource_error::ResourceErrorKind, utils::parse_type, variant::ModelVariant};

pub struct Multipart {
    cases: Vec<Case>,
}

pub struct Case {
    when: Option<When>,
    apply: Apply,
}
impl Case {
    pub fn check(&self, block_properties: &str) -> bool {
        self.when
            .as_ref()
            .is_none_or(|when| when.check(block_properties))
    }
}
#[derive(Debug, Clone)]

pub struct Apply {
    variant: ModelVariant,
}

#[derive(Debug, Clone)]
pub struct TestProperties {
    names_values: Vec<(String, String)>,
}

#[derive(Debug, Clone)]
pub enum When {
    OrCase(Vec<TestProperties>),
    AndCase(Vec<TestProperties>),
    SingleCase(TestProperties),
}
impl When {
    pub fn check(&self, block_properties: &str) -> bool {
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

impl Multipart {
    pub fn try_from_json(value: &Value) -> Result<Self, ResourceErrorKind> {
        let cases = parse_type::<Vec<Value>>(value)?
            .iter()
            .map(Case::try_from_json)
            .collect::<Result<Vec<_>, ResourceErrorKind>>()?;

        Ok(Multipart { cases })
    }

    pub(crate) fn load_models(&self, properties: &str) -> Vec<ModelVariant> {
        todo!()
    }
}

impl Case {
    pub fn try_from_json(value: &Value) -> Result<Self, ResourceErrorKind> {
        let when = value
            .get("when")
            .map(|value| {
                if let Some(value) = value.get("OR") {
                    //"Or" and "And" case are a list of test states, which are a json object containing an indeterminate number of fields in the format "state name" : "state_value(s)"
                    let test_states = Case::collect_test_states(value)?;
                    Ok(When::OrCase(test_states))
                } else if let Some(value) = value.get("AND") {
                    let test_states = Case::collect_test_states(value)?;
                    Ok(When::AndCase(test_states))
                } else {
                    let test_states = Case::collect_test_states(value)?;
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

        let variant = ModelVariant::from_json_value(try_get_field(value, "apply")?)?;
        let apply = Apply { variant };

        Ok(Case { when, apply })
    }

    pub fn collect_test_states(value: &Value) -> Result<Vec<TestProperties>, ResourceErrorKind> {
        parse_type::<Vec<Value>>(value)?
            .iter()
            .map(|value| {
                let names_values = parse_type::<Map<String, Value>>(value)?
                    .iter()
                    .map(|(state_name, state_value)| {
                        let values_str = parse_type::<&str>(state_value)?;
                        Ok((String::from(state_name), String::from(values_str)))
                    })
                    .collect::<Result<Vec<_>, ResourceErrorKind>>()?; // if we fail parsing at any point we want to just return an error for the whole thing
                Ok(TestProperties { names_values })
            })
            .collect::<Result<Vec<_>, ResourceErrorKind>>()
    }
}
