use std::marker::PhantomData;

use bumpalo::{collections::CollectIn, Bump};
use log::error;
use serde_json::{Map, Value};

use super::{
    block_models::{BlockModel, BlockRotation, ASSET_PATH},
    block_states::BlockState,
    multipart::TestStates,
    resource::ResourcePath,
    resource_error::ResourceErrorKind,
    utils::{parse_type, try_get_field},
};
use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
#[derive(Debug, Clone)]

pub struct Weight(f32);

impl TryFrom<f32> for Weight {
    type Error = ResourceErrorKind;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            return Err(ResourceErrorKind::InvalidField(format!(
                "NaN or infinite value for weight"
            )));
        }
    }
}

impl TryFrom<&Value> for Weight {
    type Error = ResourceErrorKind;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        Ok(parse_type::<f32>(value)?.try_into()?)
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
    type Error = ResourceErrorKind;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        Ok(UvLock(parse_type::<bool>(value)?))
    }
}

#[derive(Debug, Clone)]
pub enum ModelVariant<'a> {
    SingleModel(VariantEntry<'a>),
    ModelArray(BumpVec<'a, VariantEntry<'a>>),
}

#[derive(Debug)]

pub struct Variants<'a> {
    variants: BumpVec<'a, (BlockState<'a>, ModelVariant<'a>)>,
}

#[derive(Debug, Clone)]
pub struct VariantEntry<'a> {
    pub model_path: ResourcePath<'a>,
    pub rotation_x: Option<BlockRotation>,
    pub rotation_y: Option<BlockRotation>,
    pub uv_lock: Option<UvLock>,
    pub weight: Option<Weight>,
}

impl<'a> Variants<'a> {
    pub fn get_model_variants(&self, block_state: &BlockState<'a>) -> Vec<ModelVariant> {
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

impl<'a> ModelVariant<'a> {
    pub fn from_json_value(value: &Value, bump: &'a Bump) -> Result<Self, ResourceErrorKind> {
        match value.is_array() {
            true => {
                let entries = parse_type::<Vec<Value>>(value)
                    .unwrap()
                    .iter()
                    .map(|entry| VariantEntry::from_json_value(entry, bump))
                    .collect_in::<Result<BumpVec<'a, _>, ResourceErrorKind>>(bump)?;

                Ok(ModelVariant::ModelArray(entries))
            }
            false => Ok(ModelVariant::SingleModel(VariantEntry::from_json_value(
                value, bump,
            )?)),
        }
    }
}
impl<'a> Variants<'a> {
    pub(crate) fn from_json_value(
        value: &Value,
        bump: &'a Bump,
    ) -> Result<Self, ResourceErrorKind> {
        let model_variants: BumpVec<(BlockState, ModelVariant)> =
            parse_type::<Map<String, Value>>(value)?
                .iter()
                .map(|(properties, model)| {
                    let test_state = BlockState {
                        properties: BumpString::from_str_in(&properties, &bump),
                    };
                    let model = ModelVariant::from_json_value(model, &bump)?;
                    Ok((test_state, model))
                })
                .collect_in::<Result<BumpVec<'a, (BlockState, ModelVariant)>, _>>(&bump)?;

        return Ok(Variants {
            variants: model_variants,
        });
    }
}

impl<'a> VariantEntry<'a> {
    fn from_json_value(value: &Value, bump: &'a Bump) -> Result<Self, ResourceErrorKind> {
        let model_path_str = parse_type::<&str>(try_get_field(value, "model")?)?;
        let model_path = ResourcePath::BlockModel(BumpString::from_str_in(model_path_str, bump));

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
            model_path,
            rotation_x: x_rotation,
            rotation_y: y_rotation,
            uv_lock: uv_lock,
            weight,
        })
    }
}
