use std::{fs, sync::Arc};

use fxhash::FxBuildHasher;
use hashbrown::HashMap;
use lasso::{Spur, ThreadedRodeo};
use serde_json::Value;
use smol_str::SmolStr;

use super::{
    block_display::BlockDisplay,
    block_element::BlockElement,
    block_texture::{BlockTextures, TexPath, TexVar, TextureVariable},
};
#[derive(Debug, Clone)]
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
pub type AmbientOcclusion = bool;
pub type Parent = Spur;
#[derive(Debug)]
pub struct IntermediateBlockModel {
    pub parent: Option<Parent>,
    pub ambient_occlusion: Option<AmbientOcclusion>,
    pub displays: Option<Vec<BlockDisplay>>,
    pub textures: Option<BlockTextures>,
    pub elements: Option<Vec<BlockElement>>,
}

#[derive(Debug, Clone)]
pub struct BlockModel {
    pub ambient_occlusion: AmbientOcclusion,
    pub displays: Vec<BlockDisplay>,
    textures: BlockTextures,
    pub elements: Vec<BlockElement>,
}
impl BlockModel {
    pub fn get_textures(&self) -> &[(TexVar, TextureVariable)] {
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
    pub(crate) fn load(path: &str, rodeo: &Arc<ThreadedRodeo>) -> BlockModel {
        let tmp = IntermediateBlockModel::from_json(path, rodeo);
        let res = IntermediateBlockModel::collapse_parents(tmp, rodeo);
        res
    }
}

impl From<IntermediateBlockModel> for BlockModel {
    fn from(value: IntermediateBlockModel) -> Self {
        let IntermediateBlockModel {
            parent: _,
            ambient_occlusion,
            displays,
            textures,
            elements,
        } = value;

        Self {
            ambient_occlusion: ambient_occlusion.unwrap_or(false),
            displays: displays.unwrap_or(vec![]),
            textures: textures.expect("didn't find textures when finalizing block model"),
            elements: elements.unwrap_or(vec![]),
        }
    }
}

pub const ASSET_PATH: SmolStr = SmolStr::new_static("./test_assets/assets/");

impl IntermediateBlockModel {
    fn parent_to_path(parent_str: &str) -> SmolStr {
        let (namespace, remaining_str) = parent_str
            .split_once(":")
            .unwrap_or(("minecraft", parent_str));
        //        dbg!(namespace, remaining_str);

        let (model_type, remaining_str) = remaining_str
            .split_once("/")
            .expect("invalid path for parent");
        //        dbg!(model_type, remaining_str);
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
    fn collapse_parents(model: IntermediateBlockModel, rodeo: &Arc<ThreadedRodeo>) -> BlockModel {
        if model.parent.is_some() {
            let parent = model.parent.as_ref().unwrap();
            let parent_path = IntermediateBlockModel::parent_to_path(rodeo.resolve(parent));
            let mut parent = IntermediateBlockModel::from_json(&parent_path, rodeo);
            if model.elements.is_some() {
                parent.elements = model.elements;
            }
            if model.textures.is_some() {
                parent.textures = model.textures;
            }
            if model.displays.is_some() {
                parent.displays = model.displays;
            }
            return parent.into();
        } else {
            return model.into();
        }
    }

    fn from_json(path: &str, rodeo: &Arc<ThreadedRodeo>) -> IntermediateBlockModel {
        let json =
            fs::read_to_string(path).expect(format!("failed to read json file: {}", path).as_str());
        let value: Value = serde_json::from_str(&json).expect("failed to parse json");

        let parent = value
            .get("parent")
            .map(|value| rodeo.get_or_intern(value.as_str().expect("parent was not str")));

        let ambient_occlusion = value
            .get("ambientocclusion")
            .map(|value| value.as_bool().expect("ambient occlusion was not bool"));

        let displays = value
            .get("display")
            .map(|value| BlockDisplay::parse_display(value));

        let textures = value
            .get("textures")
            .map(|value| BlockTextures::parse_from_json(value, rodeo));

        let elements = value
            .get("elements")
            .map(|value| BlockElement::parse_elements(value, rodeo));
        let result = IntermediateBlockModel {
            parent,
            ambient_occlusion,
            displays,
            textures,
            elements,
        };
        result
    }
}
