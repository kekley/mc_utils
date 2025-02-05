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
    block_models::{BlockModel, BlockRotation, ASSET_PATH},
    block_states::InternalBlockState,
    block_texture::Uv,
    resource::BlockStates,
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
    variants: FxHashMap<InternalBlockState, ModelVariant>,
}

#[derive(Debug, Clone)]
pub struct VariantEntry {
    pub model: BlockModel,
    pub rotation_x: Option<BlockRotation>,
    pub rotation_y: Option<BlockRotation>,
    pub uv_lock: Option<UvLock>,
    pub weight: Option<Weight>,
}

impl Variants {
    pub fn get(&self, block_state: &InternalBlockState) -> Vec<ModelVariant> {
        dbg!(&block_state);
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
    pub fn from_json_value(value: &Value, rodeo: &ThreadedRodeo) -> Self {
        match value.is_array() {
            true => {
                let entries = value
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|entry| VariantEntry::from_json_value(entry, rodeo))
                    .collect::<Vec<_>>();
                ModelVariant::ModelArray(entries)
            }
            false => ModelVariant::SingleModel(VariantEntry::from_json_value(value, rodeo)),
        }
    }
}
impl Variants {
    pub(crate) fn from_json_value(value: &Value, rodeo: &ThreadedRodeo) -> Self {
        let variants = value
            .as_object()
            .expect("variants was not object")
            .iter()
            .map(|(variant_properties, variant_entry)| {
                let variant_entry = ModelVariant::from_json_value(variant_entry, rodeo);
                (
                    InternalBlockState::from_str(&variant_properties, &rodeo),
                    variant_entry,
                )
            })
            .collect::<HashMap<_, _, _>>();
        return Variants { variants };
    }
}

impl VariantEntry {
    fn from_json_value(value: &Value, rodeo: &ThreadedRodeo) -> Self {
        let model = value
            .get("model")
            .expect("variant did not have model")
            .as_str()
            .expect("model was not str");
        let file_path = Variants::parse_path(model);
        let block_model = BlockModel::load(&file_path, rodeo);
        let y_rotation = value.get("y").map(|value| BlockRotation::from(value));
        let x_rotation = value.get("x").map(|value| BlockRotation::from(value));
        let uv_lock = value.get("uvlock").map(|value| UvLock::from(value));
        let weight = value.get("weight").map(|value| Weight::from(value));
        VariantEntry {
            model: block_model,
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
