use std::{fs, sync::Arc};

use bumpalo::Bump;
use log::debug;
use serde_json::Value;

use super::{
    block_display::BlockDisplay,
    block_element::BlockElement,
    block_texture::{BlockTextureMap, TextureVariableEnum},
    resource_error::{ResourceError, ResourceErrorKind},
};
#[derive(Debug, Clone, Copy, Default)]
pub struct AmbientOcclusion(bool);

impl From<bool> for AmbientOcclusion {
    fn from(value: bool) -> Self {
        Self(value)
    }
}

impl From<&bool> for AmbientOcclusion {
    fn from(value: &bool) -> Self {
        Self(*value)
    }
}

#[derive(Debug, Clone)]
pub struct BlockModelParent<'a> {
    value: bumpalo::collections::String<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum BlockRotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}
impl TryFrom<&Value> for BlockRotation {
    type Error = ResourceErrorKind;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_i64() {
            Some(int) => match int {
                0 => Ok(BlockRotation::Zero),
                90 => Ok(BlockRotation::Ninety),
                180 => Ok(BlockRotation::OneEighty),
                270 => Ok(BlockRotation::TwoSeventy),
                _ => Err(format!(
                    "Block rotations must be 0, 90, 180, or 270. Got {}",
                    value.as_i64().unwrap()
                )),
            },
            None => {
                return Err(format!(
                    "Expected int value for block rotation, json value: {}",
                    value.to_string()
                ))
            }
        }
    }
}
impl BlockRotation {
    fn rotation_x_angle(&self) -> f32 {
        match self {
            BlockRotation::Zero => 0.0,
            BlockRotation::Ninety => 90.0,
            BlockRotation::OneEighty => 180.0,
            BlockRotation::TwoSeventy => 270.0,
        }
    }
}
#[derive(Debug, Clone)]
pub struct IntermediateBlockModel<'a> {
    pub parent: Option<BlockModelParent<'a>>,
    pub ambient_occlusion: Option<AmbientOcclusion>,
    pub displays: Option<bumpalo::collections::Vec<'a, BlockDisplay>>,
    pub textures: Option<BlockTextureMap<'a>>,
    pub elements: Option<bumpalo::collections::Vec<'a, BlockElement<'a>>>,
}

#[derive(Debug, Clone)]
pub struct BlockModel<'a> {
    pub ambient_occlusion: AmbientOcclusion,
    pub displays: bumpalo::collections::Vec<'a, BlockDisplay>,
    textures: BlockTextureMap<'a>,
    pub elements: bumpalo::collections::Vec<'a, BlockElement<'a>>,
}
impl<'a> BlockModel<'a> {
    pub fn get_textures(&self) -> &BlockTextureMap {
        &self.textures
    }
    pub fn is_axis_aligned(&self) -> bool {
        self.elements
            .iter()
            .all(|element| element.is_axis_aligned())
    }
    pub fn is_cube(&self) -> bool {
        self.elements.len() == 1 && self.elements[0].is_cube()
    }
}

impl<'a> BlockModel<'a> {
    pub fn try_from_intermediate(value: &IntermediateBlockModel) -> Option<Self> {
        let IntermediateBlockModel {
            parent: _,
            ambient_occlusion,
            displays,
            textures,
            elements,
        } = value;

        Some(Self {
            ambient_occlusion: ambient_occlusion.unwrap_or_default(),
            displays: displays.clone().unwrap_or_default(),
            textures: textures.as_ref()?.clone(),
            elements: elements.clone().unwrap_or_default(),
        })
    }
}

pub const ASSET_PATH: &str = "./test_assets/assets/";

impl<'a> IntermediateBlockModel<'a> {
    pub fn parent_to_path(parent_str: &str) -> String {
        let (namespace, remaining_str) = parent_str
            .split_once(":")
            .unwrap_or(("minecraft", parent_str));
        //        dbg!(namespace, remaining_str);

        let (model_type, remaining_str) = remaining_str
            .split_once("/")
            .unwrap_or(("block", remaining_str));
        //        dbg!(model_type, remaining_str);
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

    pub fn from_json(
        path: &str,
        bump: &'a mut Bump,
    ) -> Result<IntermediateBlockModel<'a>, ResourceError> {
        debug!("loading block model from file:{}", &path);
        let json = fs::read_to_string(path)?;
        let value: Value = serde_json::from_str(&json).context("invalid json")?;
        let parent: Option<Result<BlockModelParent, ResourceErrorKind>> =
            value.get("parent").map(|value| match value.as_str() {
                Some(str) => bumpalo::collections::String::from_str_in(str, bump),
                None => Err(ResourceErrorKind::InvalidField(format!(
                    "Parent field must be a string. json value: {}",
                    value.to_string()
                ))),
            });
        let parent = parent.transpose()?;

        let ambient_occlusion = value
            .get("ambientocclusion")
            .map(|value| match value.as_bool() {
                Some(bool) => {}
                None => todo!(),
            });

        let displays = value
            .get("display")
            .map(|value| BlockDisplay::parse_display(value, bump))
            .transpose()?;

        let textures = value
            .get("textures")
            .map(|value| BlockTextureMap::parse_from_json(value, bump));

        let elements = value
            .get("elements")
            .map(|value| BlockElement::parse_elements(value, bump))
            .transpose()?;

        let result = IntermediateBlockModel {
            parent,
            ambient_occlusion,
            displays,
            textures,
            elements,
        };
        Ok(result)
    }
}
