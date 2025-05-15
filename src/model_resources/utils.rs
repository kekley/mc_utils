use glam::{Vec3, Vec4};
use serde_json::Value;

pub fn parse_vec3(value: &Value) -> Vec3 {
    let result: [f32; 3] = value
        .as_array()
        .expect("value was not array")
        .iter()
        .map(|value| value.as_f64().expect("array did not contain floats") as f32)
        .collect::<Vec<f32>>()
        .try_into()
        .expect("array was not of len 3");
    Vec3::from_array(result)
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
}
