use std::{
    collections::HashMap,
    fs::{self, File},
    io::Read,
    sync::Arc,
};

use fxhash::FxHashMap;
use lasso::{Spur, ThreadedRodeo};
use serde_json::{value, Value};
use smol_str::SmolStr;

use super::{
    block_models::{BlockRotation, ASSET_PATH},
    block_states::BlockState,
    block_texture::Uv,
};
#[derive(Debug, Clone)]

pub struct Weight(f32);

#[derive(Debug, Clone)]

pub struct UvLock(bool);

#[derive(Debug, Clone)]
pub enum ModelVariant {
    SingleModel(VariantEntry),
    ModelArray(Vec<VariantEntry>),
}

pub type BlockName = Spur;
#[derive(Debug, Clone)]

pub struct Variants {
    variants: FxHashMap<BlockState, ModelVariant>,
}

#[derive(Debug, Clone)]
pub struct VariantEntry {
    pub model_path: SmolStr,
    pub rotation_x: Option<BlockRotation>,
    pub rotation_y: Option<BlockRotation>,
    pub uv_lock: Option<UvLock>,
    pub weight: Option<Weight>,
}

impl Variants {
    pub fn get(&self, block_state: &BlockState) -> Vec<ModelVariant> {
        vec![self.variants.get(block_state).unwrap().clone()]
    }
    pub fn parse_path(model_path: &str) -> SmolStr {
        let (namespace, remaining_str) = model_path.split_once(":").unwrap_or(("", model_path));

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
}

impl ModelVariant {
    pub fn new(value: &Value) -> Self {
        match value.is_array() {
            true => {
                let entries = value
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|entry| VariantEntry::from(entry))
                    .collect::<Vec<_>>();
                ModelVariant::ModelArray(entries)
            }
            false => ModelVariant::SingleModel(VariantEntry::from(value)),
        }
    }
}
impl Variants {
    pub(crate) fn new(value: &Value, rodeo: &ThreadedRodeo) -> Self {
        let variants = value
            .as_object()
            .expect("variants was not object")
            .iter()
            .map(|(variant_properties, variant_entry)| {
                let variant_entry = ModelVariant::new(variant_entry);
                (BlockState::new(&variant_properties, &rodeo), variant_entry)
            })
            .collect::<HashMap<_, _, _>>();
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
