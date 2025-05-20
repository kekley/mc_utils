use std::{any, sync::Arc};

use anyhow::{anyhow, Context, Error};
use lasso::ThreadedRodeo;
use log::error;
use serde_json::Value;

use crate::MCResourceLoader;

use super::{
    block::InternedBlock,
    block_models::{BlockRotation, InternedBlockModel, ASSET_PATH},
    block_states::InternedBlockState,
};
#[derive(Debug, Clone)]

pub struct Weight(f32);

impl TryFrom<f32> for Weight {
    type Error = anyhow::Error;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            return Err(anyhow!("NaN or infinite value for weight"));
        }
    }
}

impl TryFrom<&f32> for Weight {
    type Error = anyhow::Error;

    fn try_from(value: &f32) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self(*value))
        } else {
            return Err(anyhow!("NaN or infinite value for weight"));
        }
    }
}

impl TryFrom<f64> for Weight {
    type Error = anyhow::Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self(value as f32))
        } else {
            return Err(anyhow!("NaN or infinite value for weight"));
        }
    }
}

impl TryFrom<&f64> for Weight {
    type Error = anyhow::Error;

    fn try_from(value: &f64) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self(*value as f32))
        } else {
            return Err(anyhow!("NaN or infinite value for weight"));
        }
    }
}

impl TryFrom<&Value> for Weight {
    type Error = anyhow::Error;
    fn try_from(value: &Value) -> Result<Self, Error> {
        Ok(value
            .as_f64()
            .context("\"weight\" field was not a number")?
            .try_into()?)
    }
}

#[derive(Debug, Clone)]

pub struct UvLock(bool);

impl From<bool> for UvLock {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl From<&bool> for UvLock {
    fn from(value: &bool) -> Self {
        Self(*value)
    }
}

impl TryFrom<&Value> for UvLock {
    type Error = anyhow::Error;
    fn try_from(value: &Value) -> Result<Self, Error> {
        Ok(UvLock(
            value
                .as_bool()
                .context("\"uvlock\" field was not bool")?
                .into(),
        ))
    }
}

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
    pub fn get_model_variants(&self, block_state: &InternedBlockState) -> Vec<ModelVariant> {
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
        if a.len() == 0 {
            error!("uh");
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
    pub fn from_json_value(value: &Value, loader: &MCResourceLoader) -> anyhow::Result<Self> {
        match value.is_array() {
            true => {
                let entries: Vec<VariantEntry> = value
                    .as_array()
                    .unwrap()
                    .iter()
                    .map(|entry| VariantEntry::from_json_value(entry, loader))
                    .collect::<Result<Vec<_>, _>>()?;
                Ok(ModelVariant::ModelArray(entries))
            }
            false => Ok(ModelVariant::SingleModel(VariantEntry::from_json_value(
                value, loader,
            )?)),
        }
    }
}
impl Variants {
    pub(crate) fn from_json_value(
        value: &Value,
        loader: &MCResourceLoader,
    ) -> anyhow::Result<Self> {
        let model_variants: Result<Vec<(InternedBlockState, ModelVariant)>, anyhow::Error> = value
            .as_object()
            .context("variants was not object")?
            .iter()
            .map(|(variant_properties, variant_entry)| {
                let variant_entry = ModelVariant::from_json_value(variant_entry, loader)?;
                Ok((
                    InternedBlockState::from_str(&variant_properties, loader),
                    variant_entry,
                ))
            })
            .collect::<Result<Vec<_>, _>>();

        return Ok(Variants {
            variants: model_variants?,
        });
    }
}

impl VariantEntry {
    fn from_json_value(value: &Value, loader: &MCResourceLoader) -> anyhow::Result<Self> {
        let model = value
            .get("model")
            .context("variant did not have model")?
            .as_str()
            .context("model was not str")?;
        let spur = loader.rodeo.get_or_intern(model);
        let block_model = loader.load_block_model(spur)?;
        let y_rotation = value
            .get("y")
            .map(|value| BlockRotation::try_from(value))
            .transpose()?;
        let x_rotation = value
            .get("x")
            .map(|value| BlockRotation::try_from(value))
            .transpose()?;
        let uv_lock = value
            .get("uvlock")
            .map(|value| UvLock::try_from(value))
            .transpose()?;
        let weight = value
            .get("weight")
            .map(|value| Weight::try_from(value))
            .transpose()?;
        Ok(VariantEntry {
            model: block_model,
            rotation_x: x_rotation,
            rotation_y: y_rotation,
            uv_lock: uv_lock,
            weight,
        })
    }
}
