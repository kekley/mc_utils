use std::{
    fs,
    sync::Arc,
};

use anyhow::{anyhow, Context, Error};
use lasso::{Spur, ThreadedRodeo};
use log::debug;
use serde_json::Value;

use super::{
    block_display::BlockDisplay,
    block_element::InternedBlockElement,
    block_texture::{BlockTextures, InternedTextureVariable, TexVar},
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

#[derive(Debug, Clone, Copy)]
pub struct BlockModelParent(pub Spur);
impl From<Spur> for BlockModelParent {
    fn from(value: Spur) -> Self {
        BlockModelParent(value)
    }
}
impl From<&Spur> for BlockModelParent {
    fn from(value: &Spur) -> Self {
        BlockModelParent(*value)
    }
}

impl Into<Spur> for BlockModelParent {
    fn into(self) -> Spur {
        self.0 as Spur
    }
}
impl<'a> Into<&'a Spur> for &'a BlockModelParent {
    fn into(self) -> &'a Spur {
        &(self.0) as &Spur
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
    type Error = anyhow::Error;
    fn try_from(value: &Value) -> Result<Self, Error> {
        match value.as_i64().context(format!(
            "Expected int value for block rotation, json value: {}",
            value.to_string()
        ))? {
            0 => Ok(BlockRotation::Zero),
            90 => Ok(BlockRotation::Ninety),
            180 => Ok(BlockRotation::OneEighty),
            270 => Ok(BlockRotation::TwoSeventy),
            _ => Err(anyhow!(format!(
                "Block rotations must be 0, 90, 180, or 270. Got {}",
                value.as_i64().unwrap()
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
pub struct IntermediateBlockModel {
    pub parent: Option<BlockModelParent>,
    pub ambient_occlusion: Option<AmbientOcclusion>,
    pub displays: Option<Vec<BlockDisplay>>,
    pub textures: Option<BlockTextures>,
    pub elements: Option<Vec<InternedBlockElement>>,
}

#[derive(Debug, Clone)]
pub struct InternedBlockModel {
    pub ambient_occlusion: AmbientOcclusion,
    pub displays: Vec<BlockDisplay>,
    textures: BlockTextures,
    pub elements: Vec<InternedBlockElement>,
}
impl InternedBlockModel {
    pub fn get_textures(&self) -> &[(TexVar, InternedTextureVariable)] {
        self.textures.get_all()
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

impl InternedBlockModel {
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

impl IntermediateBlockModel {
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
        rodeo: &Arc<ThreadedRodeo>,
    ) -> anyhow::Result<IntermediateBlockModel> {
        debug!("loading block model from file:{}", &path);

        let json = fs::read_to_string(path)?;
        let value: Value = serde_json::from_str(&json).context("invalid json")?;
        let parent: Option<Result<BlockModelParent, anyhow::Error>> =
            value.get("parent").map(|value| {
                Ok(rodeo
                    .get_or_intern(value.as_str().context("parent was not str")?)
                    .into())
            });
        let parent = parent.transpose()?;

        let ambient_occlusion = value.get("ambientocclusion").map(|value| {
            value
                .as_bool()
                .expect("ambient occlusion was not bool")
                .into()
        });

        let displays = value
            .get("display")
            .map(|value| BlockDisplay::parse_display(value))
            .transpose()?;

        let textures = value
            .get("textures")
            .map(|value| BlockTextures::parse_from_json(value, rodeo));

        let elements = value
            .get("elements")
            .map(|value| InternedBlockElement::parse_elements(value, rodeo))
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
