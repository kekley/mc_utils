use std::{
    fs::{self, File},
    io::Read,
};

use fxhash::FxHashMap;
use serde_json::Value;
use smol_str::SmolStr;

use super::{
    block_models::{BlockRotation, ASSET_PATH},
    block_texture::Uv,
};
pub struct Weight(f32);

pub struct UvLock(bool);

pub enum Variant {
    SingleModel(VariantEntry),
    ModelArray(Vec<VariantEntry>),
}
pub struct Variants {
    variants: FxHashMap<SmolStr, Variant>,
}

pub struct VariantEntry {
    pub model_path: SmolStr,
    pub rotation_x: Option<BlockRotation>,
    pub rotation_y: Option<BlockRotation>,
    pub uv_lock: Option<UvLock>,
    pub weight: Option<Weight>,
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
    pub fn from_json(path: &str) -> Variants {
        let contents = fs::read_to_string(path).expect("could not read json file");

        let value: Value = serde_json::from_str(&contents).expect("could_not parse_json");
        if let Some(value) = value.get("variants") {
            let variants = Variants::from(value);
            return variants;
        } else {
            let multipart = value
                .get("multipart")
                .expect("file was not variant or multipart");
            todo!()
        }
    }
}

impl From<&Value> for Variants {
    fn from(value: &Value) -> Self {
        let variants = value
            .as_object()
            .expect("variants was not object")
            .iter()
            .map(|(variant_name, variant_entry)| {
                let variant = match variant_entry.is_array() {
                    true => {
                        let entries = variant_entry
                            .as_array()
                            .unwrap()
                            .iter()
                            .map(|entry| VariantEntry::from(entry))
                            .collect::<Vec<_>>();
                        Variant::ModelArray(entries)
                    }
                    false => Variant::SingleModel(VariantEntry::from(variant_entry)),
                };
                (SmolStr::from(variant_name), variant)
            })
            .collect::<FxHashMap<_, _>>();
        return Variants { variants };
    }
}

impl From<&Value> for VariantEntry {
    fn from(value: &Value) -> Self {
        let model = value.as_str().expect("model was not string");
        let y_rotation = value.get("y").map(|value| BlockRotation::from(value));
        let x_rotation = value.get("x").map(|value| BlockRotation::from(value));
        let uv_lock = value.get("uvlock").map(|value| UvLock::from(value));
        let weight = value.get("weight").map(|value| Weight::from(value));
        VariantEntry {
            model_path: SmolStr::from(model),
            rotation_x: x_rotation,
            rotation_y: y_rotation,
            uv_lock: uv_lock,
            weight,
        }
    }
}

impl From<&Value> for UvLock {
    fn from(value: &Value) -> Self {
        UvLock(value.as_bool().expect("uvlock was not bool"))
    }
}

impl From<&Value> for Weight {
    fn from(value: &Value) -> Self {
        Weight(value.as_f64().expect("weight was not a number") as f32)
    }
}
