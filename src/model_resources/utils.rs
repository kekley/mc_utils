use serde_json::Value;
use std::{any::type_name, mem::MaybeUninit};

use super::resource_error::ResourceErrorKind;
/*
pub fn parse_f32_3(value: &Value) -> anyhow::Result<[f32; 3]> {
    let values = value.as_array().context("Value was not an array")?;
    if values.len() != 3 {
        return Err(anyhow!(""));
    }
    values.as_slice()
}
pub fn parse_vec4(value: &Value) -> Vec4 {
    let result: [f32; 4] = value
        .as_array()
        .expect("value was not array")
        .iter()
        .map(|value| value.as_f64().expect("array did not contain floats") as f32)
        .collect::<Vec<f32>>()
        .try_into()
        .expect("array was not of len 4");
    Vec4::from_array(result)
} */

pub fn parse_array<T: for<'a> serde::de::Deserialize<'a>, const N: usize>(
    value: &Value,
) -> Result<[T; N], ResourceErrorKind> {
    let values: &Vec<Value> = match value.as_array() {
        Some(vec) => vec,
        None => {
            return Err(ResourceErrorKind::InvalidField(
                "Attempted to parse a non array value as an array".to_string(),
            ))
        }
    };
    if values.len() != N {
        return Err(ResourceErrorKind::InvalidField(format!(
            "Expected array length of {N}, got {} instead",
            values.len()
        )));
    }
    let mut uninit_array: [MaybeUninit<T>; N] = [const { MaybeUninit::uninit() }; N];

    for (i, value) in values.iter().enumerate() {
        uninit_array[i].write(match T::deserialize(value) {
            Ok(t) => t,
            Err(err) => {
                return Err(ResourceErrorKind::InvalidField(format!(
                    "Could not deserialize values in array: json error {err}"
                )))
            }
        });
    }
    Ok(uninit_array.map(|uninit| unsafe { uninit.assume_init() }))
}

pub fn try_get_field<'a>(
    value: &'a Value,
    field_name: &str,
) -> Result<&'a Value, ResourceErrorKind> {
    match value.get(field_name) {
        Some(field) => Ok(field),
        None => Err(ResourceErrorKind::MissingField(format!(
            "Field name \"{field_name}\" was missing"
        ))),
    }
}

pub fn get_optional_field<'a>(value: &'a Value, field_name: &str) -> Option<&'a Value> {
    match value.get(field_name) {
        Some(field) => Some(field),
        None => None,
    }
}

pub fn parse_type<'a, T: serde::de::Deserialize<'a>>(
    value: &'a Value,
) -> Result<T, ResourceErrorKind> {
    match T::deserialize(value) {
        Ok(t) => Ok(t),
        Err(err) => Err(ResourceErrorKind::InvalidField(format!(
            "Error parsing {}, json error: {}",
            type_name::<T>(),
            err
        ))),
    }
}
