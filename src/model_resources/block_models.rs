use std::fs;
use tracing::debug;

use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::Bump;
use serde_json::Value;

use crate::resource_error::create_resource_error;
use crate::utils::get_optional_field;

use super::{
    block_display::BlockDisplay,
    block_element::BlockElement,
    block_texture::BlockTextureMap,
    resource_error::{ResourceError, ResourceErrorKind},
    utils::parse_type,
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

impl<'a> BlockModelParent<'a> {
    fn try_from_in(value: &Value, bump: &'a Bump) -> Result<Self, ResourceErrorKind> {
        let str = parse_type::<&str>(value)?;

        Ok(BlockModelParent {
            value: BumpString::from_str_in(str, bump),
        })
    }
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
        let int = parse_type::<i64>(value)?;
        match int {
            0 => Ok(BlockRotation::Zero),
            90 => Ok(BlockRotation::Ninety),
            180 => Ok(BlockRotation::OneEighty),
            270 => Ok(BlockRotation::TwoSeventy),
            _ => Err(ResourceErrorKind::InvalidField(format!(
                "Block rotations must be 0, 90, 180, or 270. Got {}",
                int
            ))),
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
    pub displays: Option<BumpVec<'a, BlockDisplay>>,
    pub textures: Option<BlockTextureMap<'a>>,
    pub elements: Option<BumpVec<'a, BlockElement<'a>>>,
}

#[derive(Debug, Clone)]
pub struct BlockModel<'a> {
    pub ambient_occlusion: AmbientOcclusion,
    pub displays: BumpVec<'a, BlockDisplay>,
    textures: BlockTextureMap<'a>,
    pub elements: BumpVec<'a, BlockElement<'a>>,
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
    pub fn try_from_intermediate(value: &'a IntermediateBlockModel) -> Option<Self> {
        let IntermediateBlockModel {
            parent: _,
            ambient_occlusion,
            displays,
            textures,
            elements,
        } = value;

        Some(Self {
            ambient_occlusion: ambient_occlusion.unwrap_or_default(),
            displays: displays.clone().unwrap(),
            textures: textures.as_ref()?.clone(),
            elements: elements.clone().unwrap(),
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
        let file_string = create_resource_error(path, || fs::read_to_string(path))?;

        let json_value =
            create_resource_error(path, || serde_json::from_str::<Value>(&file_string))?;

        let parent_field = get_optional_field(&json_value, "parent");

        let parent = match parent_field {
            Some(parent) => Some(create_resource_error(path, || {
                BlockModelParent::try_from_in(parent, bump)
            })?),
            None => None,
        };
        let ambient_occlusion_field = get_optional_field(&json_value, "ambientocclusion");

        let ambient_occlusion = match ambient_occlusion_field {
            Some(value) => Some(create_resource_error(path, || parse_type::<bool>(value))?.into()),
            None => None,
        };

        let displays_field = get_optional_field(&json_value, "display");

        let displays = match displays_field {
            Some(value) => Some(create_resource_error(path, || {
                BlockDisplay::parse_display(value, bump)
            })?),
            None => None,
        };

        let textures_field = get_optional_field(&json_value, "textures");
        let textures = match textures_field {
            Some(value) => Some(create_resource_error(path, || {
                BlockTextureMap::try_from_json(value, bump)
            })?),
            None => None,
        };

        let elements_field = get_optional_field(&json_value, "elements");

        let elements = match elements_field {
            Some(value) => Some(create_resource_error(path, || {
                BlockElement::parse_elements(value, bump)
            })?),
            None => None,
        };

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
