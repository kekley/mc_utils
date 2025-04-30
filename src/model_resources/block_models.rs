use std::{fs, sync::Arc};

use glam::{Affine3A, Quat, Vec3};
use lasso::{Spur, ThreadedRodeo};
use serde_json::Value;
use smol_str::SmolStr;

use super::{
    block_display::BlockDisplay,
    block_element::InternedBlockElement,
    block_texture::{BlockTextures, InternedTextureVariable, TexVar},
};
#[derive(Debug, Clone, Copy)]
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
impl BlockRotation {
    fn rotation_x_angle(&self) -> f32 {
        match self {
            BlockRotation::Zero => 0.0,
            BlockRotation::Ninety => -90.0,
            BlockRotation::OneEighty => -180.0,
            BlockRotation::TwoSeventy => -270.0,
        }
    }

    fn rotation_y_angle(&self) -> f32 {
        match self {
            BlockRotation::Zero => 0.0,
            BlockRotation::Ninety => -90.0,
            BlockRotation::OneEighty => -180.0,
            BlockRotation::TwoSeventy => -270.0,
        }
    }

    pub fn to_matrix_x(&self) -> Affine3A {
        let angle = self.rotation_x_angle();

        Affine3A::from_rotation_x(angle.to_radians())
    }
    pub fn to_matrix_y(&self) -> Affine3A {
        let angle = self.rotation_x_angle();

        Affine3A::from_rotation_y(angle.to_radians())
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
    pub(crate) fn load(path: &str, rodeo: &Arc<ThreadedRodeo>) -> InternedBlockModel {
        let tmp = IntermediateBlockModel::from_json(path, rodeo);
        let res = IntermediateBlockModel::collapse_parents(tmp, rodeo);
        res
    }
}

impl From<IntermediateBlockModel> for InternedBlockModel {
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
    fn collapse_parents(
        model: IntermediateBlockModel,
        rodeo: &Arc<ThreadedRodeo>,
    ) -> InternedBlockModel {
        if model.parent.is_some() {
            let parent = model.parent.as_ref().unwrap();
            let parent_path = IntermediateBlockModel::parent_to_path(rodeo.resolve(parent));
            let mut parent = IntermediateBlockModel::from_json(&parent_path, rodeo);
            if model.elements.is_some() {
                parent.elements = model.elements;
            }
            if model.textures.is_some() {
                if parent.textures.is_some() {
                    parent
                        .textures
                        .as_mut()
                        .unwrap()
                        .combine(model.textures.unwrap());
                } else {
                    parent.textures = model.textures;
                }
            }
            if model.displays.is_some() {
                parent.displays = model.displays;
            }

            return Self::collapse_parents(parent, rodeo);
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
            .map(|value| InternedBlockElement::parse_elements(value, rodeo));

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
