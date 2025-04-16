use std::sync::Arc;

use fxhash::{FxBuildHasher, FxHashMap, FxHasher};
use glam::Vec4;
use hashbrown::HashMap;
use lasso::{Spur, ThreadedRodeo};
use serde_json::Value;
use smol_str::SmolStr;

use super::{block_models::ASSET_PATH, utils::parse_vec4};

#[derive(Debug, Clone)]
pub struct Uv {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
}

impl Uv {
    pub fn new(x1: f32, x2: f32, y1: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }
    pub fn to_vec4(&self) -> Vec4 {
        let Uv { x1, y1, x2, y2 } = self;
        Vec4::new(*x1, *y1, *x2, *y2)
    }
}

impl From<&Value> for Uv {
    fn from(value: &Value) -> Uv {
        Uv::from(parse_vec4(value).to_array())
    }
}
impl From<[f32; 4]> for Uv {
    fn from(value: [f32; 4]) -> Self {
        Uv {
            x1: value[0],
            y1: value[1],
            x2: value[2],
            y2: value[3],
        }
    }
}
pub type TexVar = Spur;
pub type TexPath = Spur;
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TextureVariable {
    Variable(TexVar),
    ResourcePath(TexPath),
}

impl TextureVariable {
    pub fn get_inner(&self) -> Spur {
        match self {
            TextureVariable::Variable(key) => *key,
            TextureVariable::ResourcePath(key) => *key,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockTextures {
    pub textures: Vec<(TexVar, TextureVariable)>,
}

impl BlockTextures {
    pub fn get_all(&self) -> &[(TexVar, TextureVariable)] {
        &self.textures
    }
    pub fn get_keys(&self) -> Vec<TexVar> {
        self.textures.iter().map(|entry| entry.0).collect()
    }

    fn path_inner(variable: &TextureVariable, rodeo: &ThreadedRodeo) -> SmolStr {
        let path_str = rodeo.resolve(&variable.get_inner());
        let (namespace, remaining_str) = path_str.split_once(":").unwrap_or(("", path_str));

        let (texture_type, remaining_str) = remaining_str
            .split_once("/")
            .expect("invalid path for texture");
        SmolStr::new(
            ASSET_PATH.to_string()
                + namespace
                + "/"
                + "textures/"
                + texture_type
                + "/"
                + remaining_str
                + ".png",
        )
    }

    pub fn parse_from_json(value: &Value, rodeo: &Arc<ThreadedRodeo>) -> Self {
        let obj = value.as_object().expect("textures was not object");
        let textures = obj
            .iter()
            .map(|(var1, var2)| {
                let name = rodeo.get_or_intern(var1);
                let texture = TextureVariable::parse_from_json_value(var2, rodeo);
                (name, texture)
            })
            .collect();
        BlockTextures { textures }
    }
}

impl TextureVariable {
    pub(crate) fn parse_from_json_value(value: &Value, rodeo: &Arc<ThreadedRodeo>) -> Self {
        let val = value.as_str().expect("texture value was not SmolStr");
        let first_char = val.chars().nth(0).expect("texture SmolStr had length of 0");
        if val.len() == 1 {
            panic!("invalid texture variable length")
        }
        if first_char == '#' {
            return TextureVariable::Variable(rodeo.get_or_intern(val));
        } else {
            return TextureVariable::ResourcePath(rodeo.get_or_intern(val));
        }
    }
}
