use anyhow::{bail, Context};
use serde_json::Value;
use std::mem::MaybeUninit;
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
) -> anyhow::Result<[T; N]> {
    let values: &Vec<Value> = value
        .as_array()
        .context("Attempted to parse a non array value as an array")?;
    if values.len() != N {
        bail!("Expected array length of {N}, got {} instead", values.len())
    }
    let mut uninit_array: [MaybeUninit<T>; N] = [const { MaybeUninit::uninit() }; N];

    for (i, value) in values.iter().enumerate() {
        uninit_array[i].write(T::deserialize(value)?);
    }
    Ok(uninit_array.map(|uninit| unsafe { uninit.assume_init() }))
}
