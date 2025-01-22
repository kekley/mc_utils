use std::fs;

use anyhow::{anyhow, Context, Error, Ok};
use fxhash::FxHashMap;
use glam::{IVec3, Vec3, Vec4};
use rayon::vec;
use serde_json::{value, Value};

use super::{
    block_display::BlockDisplay, block_element::BlockElement, block_texture::BlockTextures,
    utils::parse_vec3,
};
#[derive(Debug)]
pub enum BlockRotation {
    Zero,
    Ninety,
    OneEighty,
    TwoSeventy,
}
impl From<&Value> for BlockRotation {
    fn from(value: &Value) -> Self {
        match value.as_i64().expect("block rotation was not int") {
            0 => BlockRotation::Zero,
            90 => BlockRotation::Ninety,
            180 => BlockRotation::OneEighty,
            270 => BlockRotation::TwoSeventy,
            _ => panic!("invalid block rotation"),
        }
    }
}

#[derive(Debug)]
pub struct BlockModel {
    parent: Option<String>,
    ambient_occlusion: Option<bool>,
    displays: Option<Vec<BlockDisplay>>,
    textures: Option<BlockTextures>,
    elements: Option<Vec<BlockElement>>,
}

const ASSET_PATH: &str = "./assets/default_resource_pack/assets/";
impl BlockModel {
    pub fn from_json(path: &str) -> BlockModel {
        let json = fs::read_to_string(path).expect("failed to read json file");
        let value: Value = serde_json::from_str(&json).expect("failed to parse json");
        let parent = value
            .get("parent")
            .map(|value| value.as_str().expect("parent was not str").to_string());

        let ambient_occlusion = value
            .get("ambientocclusion")
            .map(|value| value.as_bool().expect("ambient occlusion was not bool"));

        let displays = value
            .get("display")
            .map(|value| BlockDisplay::parse_display(value));

        let textures = value
            .get("textures")
            .map(|value| BlockTextures::parse_textures(value));

        let elements = value
            .get("elements")
            .map(|value| BlockElement::parse_elements(value));
        let result = BlockModel {
            parent,
            ambient_occlusion: ambient_occlusion,
            displays,
            textures: textures,
            elements: elements,
        };
        result
    }

    
}
