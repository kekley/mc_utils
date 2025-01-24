use std::{
    fs::{self, File},
    io::Read,
};

use serde_json::Value;
use smol_str::SmolStr;

use super::{
    block_models::{BlockRotation, ASSET_PATH},
    block_texture::Uv,
};
pub struct Weight(f32);

pub struct UvLock(bool);

pub enum VariantType {
    SingleModel(VariantModel),
    ModelArray(Vec<VariantModel>, Weight),
}
pub struct Variants {
    variants: Vec<VariantType>,
}

pub struct VariantModel {
    pub model_path: SmolStr,
    pub rotation_x: Option<BlockRotation>,
    pub rotation_y: Option<BlockRotation>,
    pub uv_lock: Option<UvLock>,
}

impl Variants {
    fn parse_path(model_path: SmolStr) -> SmolStr {
        let (namespace, remaining_str) = model_path
            .split_once(":")
            .unwrap_or(("", model_path.as_str()));

        let (model_type, remaining_str) = remaining_str
            .split_once("/")
            .expect("invalid path for model");
        SmolStr::from(
            ASSET_PATH.to_string()
                + namespace
                + "/"
                + "models/"
                + model_type
                + "/"
                + remaining_str
                + ".json",
        )
    }
    pub fn from_json(path: &str) -> VariantModel {
        let contents = fs::read_to_string(path).expect("could not read json file");

        let value: Value = serde_json::from_str(&contents).expect("could_not parse_json");
        if let Some(variants) = value.get("variants") {
            if variants.is_array() {
            } else {
            }
        } else {
            let multipart = value
                .get("multipart")
                .expect("file was not variant or multipart");
        }
        todo!()
    }
}

impl From<&Value> for Variants {
    fn from(value: &Value) -> Self {
        todo!()
    }
}

impl From<&Value> for VariantModel {
    fn from(value: &Value) -> Self {
        let model = value.as_str().expect("model was not string");
        let y_rotation = value.get("y").map(|value| BlockRotation::from(value));
        let x_rotation = value.get("x").map(|value| BlockRotation::from(value));
        let uv_lock = value.get("uvlock").map(|value| UvLock::from(value));
        VariantModel {
            model_path: SmolStr::from(model),
            rotation_x: x_rotation,
            rotation_y: y_rotation,
            uv_lock: uv_lock,
        }
    }
}

impl From<&Value> for UvLock {
    fn from(value: &Value) -> Self {
        UvLock(value.as_bool().expect("uvlock was not bool"))
    }
}
