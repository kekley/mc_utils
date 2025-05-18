use std::sync::Arc;

use lasso::ThreadedRodeo;
use serde_json::Value;

use crate::MCResourceLoader;

use super::{
    block_models::{BlockRotation, InternedBlockModel, ASSET_PATH},
    block_states::InternedBlockState,
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

#[derive(Debug, Clone)]

pub struct Variants {
    variants: Vec<(InternedBlockState, ModelVariant)>,
}

#[derive(Debug, Clone)]
pub struct VariantEntry {
    pub model: InternedBlockModel,
    pub rotation_x: Option<BlockRotation>,
    pub rotation_y: Option<BlockRotation>,
    pub uv_lock: Option<UvLock>,
    pub weight: Option<Weight>,
}

impl Variants {
    pub fn get_model(&self, block_state: &InternedBlockState) -> Vec<ModelVariant> {
        //dbg!(&block_state);

        let mut a: Vec<_> = self
            .variants
            .iter()
            .filter_map(|(block_state_, model)| {
                if block_state == block_state_ {
                    Some(model.clone())
                } else {
                    None
                }
            })
            .collect();

        if a.len() == 0 && self.variants.len() != 0 {
            a.push(self.variants[0].1.clone());
        }
        a
    }
    pub fn parse_path(model_path: &str) -> String {
        let (namespace, remaining_str) = model_path.split_once(":").unwrap_or(("", model_path));

        let (model_type, remaining_str) = remaining_str
            .split_once("/")
            .unwrap_or(("block", remaining_str));
        String::from(
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
    pub fn from_json_value(value: &Value, loader: &MCResourceLoader) -> Option<Self> {
        match value.is_array() {
            true => {
                let entries = value
                    .as_array()
                    .unwrap()
                    .iter()
                    .filter_map(|entry| VariantEntry::from_json_value(entry, loader))
                    .collect::<Vec<_>>();
                Some(ModelVariant::ModelArray(entries))
            }
            false => Some(ModelVariant::SingleModel(VariantEntry::from_json_value(
                value, loader,
            )?)),
        }
    }
}
impl Variants {
    pub(crate) fn from_json_value(value: &Value, loader: &MCResourceLoader) -> Self {
        let variants = value
            .as_object()
            .expect("variants was not object")
            .iter()
            .filter_map(|(variant_properties, variant_entry)| {
                let variant_entry = ModelVariant::from_json_value(variant_entry, loader);
                if variant_entry.is_none() {
                    None
                } else {
                    Some((
                        InternedBlockState::from_str(&variant_properties, loader),
                        variant_entry.unwrap(),
                    ))
                }
            })
            .collect();
        return Variants { variants };
    }
}

impl VariantEntry {
    fn from_json_value(value: &Value, loader: &MCResourceLoader) -> Option<Self> {
        let model = value
            .get("model")
            .expect("variant did not have model")
            .as_str()
            .expect("model was not str");
        let spur = loader.rodeo.get_or_intern(model);
        let block_model = loader.load_block_model(spur)?;
        let y_rotation = value.get("y").map(|value| BlockRotation::from(value));
        let x_rotation = value.get("x").map(|value| BlockRotation::from(value));
        let uv_lock = value.get("uvlock").map(|value| UvLock::from(value));
        let weight = value.get("weight").map(|value| Weight::from(value));
        Some(VariantEntry {
            model: block_model,
            rotation_x: x_rotation,
            rotation_y: y_rotation,
            uv_lock: uv_lock,
            weight,
        })
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
