use serde_json::{Map, Value};
use tracing::error;

use super::{
    block_models::{BlockRotation, ASSET_PATH},
    resource::ResourcePath,
    resource_error::ResourceErrorKind,
    utils::{parse_type, try_get_field},
};
#[derive(Debug, Clone)]

pub struct Weight(f32);

impl TryFrom<f32> for Weight {
    type Error = ResourceErrorKind;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if value.is_finite() {
            Ok(Self(value))
        } else {
            Err(ResourceErrorKind::InvalidField(
                "NaN or infinite value for weight".to_string(),
            ))
        }
    }
}

impl TryFrom<&Value> for Weight {
    type Error = ResourceErrorKind;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        parse_type::<f32>(value)?.try_into()
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
pub enum ModelVariant {
    SingleModel(VariantEntry),
    ModelArray(Vec<VariantEntry>),
}

#[derive(Debug)]

pub struct Variants {
    variants: Vec<(String, ModelVariant)>,
}

#[derive(Debug, Clone)]
pub struct VariantEntry {
    pub model_path: ResourcePath,
    pub rotation_x: Option<BlockRotation>,
    pub rotation_y: Option<BlockRotation>,
    pub uv_lock: Option<UvLock>,
    pub weight: Option<Weight>,
}

impl Variants {
    pub fn get_model_variants(&self, block_properties: &str) -> Vec<ModelVariant> {
        //dbg!(&block_state);

        let mut a: Vec<_> = self
            .variants
            .iter()
            .filter_map(|(block_state_, model)| {
                if block_properties == block_state_ {
                    Some(model.clone())
                } else {
                    None
                }
            })
            .collect();

        if a.is_empty() && !self.variants.is_empty() {
            a.push(self.variants[0].1.clone());
        }
        if a.is_empty() {
            error!("uh");
        }
        a
    }
    pub fn parse_path(model_path: &str) -> String {
        let (namespace, remaining_str) = model_path.split_once(":").unwrap_or(("", model_path));

        let (model_type, remaining_str) = remaining_str
            .split_once("/")
            .unwrap_or(("block", remaining_str));
        let mut path = ASSET_PATH.to_string();
        path.push_str(namespace);
        path.push('/');
        path.push_str("models/");
        path.push_str(model_type);
        path.push('/');
        path.push_str(remaining_str);
        path.push_str(".json");
        path
    }
}

impl ModelVariant {
    pub fn from_json_value(value: &Value) -> Result<Self, ResourceErrorKind> {
        match value.is_array() {
            true => {
                let entries = parse_type::<Vec<Value>>(value)
                    .unwrap()
                    .iter()
                    .map(|entry| VariantEntry::from_json_value(entry))
                    .collect::<Result<Vec<_>, ResourceErrorKind>>()?;

                Ok(ModelVariant::ModelArray(entries))
            }
            false => Ok(ModelVariant::SingleModel(VariantEntry::from_json_value(
                value,
            )?)),
        }
    }
}
impl Variants {
    pub(crate) fn from_json_value(value: &Value) -> Result<Self, ResourceErrorKind> {
        let model_variants: Vec<(String, ModelVariant)> = parse_type::<Map<String, Value>>(value)?
            .iter()
            .map(|(properties, model)| {
                let test_state = String::from(properties);
                let model = ModelVariant::from_json_value(model)?;
                Ok((test_state, model))
            })
            .collect::<Result<Vec<(String, ModelVariant)>, ResourceErrorKind>>()?;

        Ok(Variants {
            variants: model_variants,
        })
    }
}

impl VariantEntry {
    fn from_json_value(value: &Value) -> Result<Self, ResourceErrorKind> {
        let model_path_str = parse_type::<&str>(try_get_field(value, "model")?)?;
        let model_path = ResourcePath::BlockModel(String::from(model_path_str));

        let y_rotation = value.get("y").map(BlockRotation::try_from).transpose()?;
        let x_rotation = value.get("x").map(BlockRotation::try_from).transpose()?;
        let uv_lock = value.get("uvlock").map(UvLock::try_from).transpose()?;
        let weight = value.get("weight").map(Weight::try_from).transpose()?;
        Ok(VariantEntry {
            model_path,
            rotation_x: x_rotation,
            rotation_y: y_rotation,
            uv_lock,
            weight,
        })
    }
}
