use std::{array, sync::Arc};

use lasso::{Spur, ThreadedRodeo};
use serde_json::Value;

use super::{block_models::ASSET_PATH, utils::parse_array};

#[repr(C)]
#[derive(Debug, Clone)]
pub struct Uv {
    pub x1: f32,
    pub y1: f32,
    pub x2: f32,
    pub y2: f32,
}

impl Uv {
    pub fn new(x1: f32, x2: f32, y1: f32, y2: f32) -> Self {
        Self { x1, y1, x2, y2 }
    }
}

impl TryFrom<&Value> for Uv {
    type Error = anyhow::Error;
    //expects an array of f32 of length 4
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let a = parse_array::<f32, 4>(value)?;
        Ok(Uv::from(a))
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
pub enum InternedTextureVariable {
    Variable(TexVar),
    ResourcePath(TexPath),
}

impl InternedTextureVariable {
    pub fn get_inner(&self) -> Spur {
        match self {
            InternedTextureVariable::Variable(key) => *key,
            InternedTextureVariable::ResourcePath(key) => *key,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockTextures {
    pub textures: Vec<(TexVar, InternedTextureVariable)>,
}

impl BlockTextures {
    pub fn get_all(&self) -> &[(TexVar, InternedTextureVariable)] {
        &self.textures
    }
    pub fn combine(&mut self, textures: BlockTextures) {
        for texture in &textures.textures {
            if !self.textures.contains(&texture) {
                self.textures.push(texture.clone());
            }
        }
    }
    pub fn get_keys(&self) -> Vec<TexVar> {
        self.textures.iter().map(|entry| entry.0).collect()
    }

    fn path_inner(variable: &InternedTextureVariable, rodeo: &ThreadedRodeo) -> String {
        let path_str = rodeo.resolve(&variable.get_inner());
        let (namespace, remaining_str) = path_str.split_once(":").unwrap_or(("", path_str));

        let (texture_type, remaining_str) = remaining_str
            .split_once("/")
            .expect("invalid path for texture");
        String::from(
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
        let json_object = value.as_object().expect("textures was not object");
        let textures = json_object
            .iter()
            .map(|(var1, var2)| {
                let name = rodeo.get_or_intern(var1);
                let texture = InternedTextureVariable::parse_from_json_value(var2, rodeo);
                (name, texture)
            })
            .collect();
        BlockTextures { textures }
    }
}

impl InternedTextureVariable {
    pub(crate) fn parse_from_json_value(value: &Value, rodeo: &Arc<ThreadedRodeo>) -> Self {
        let val = value.as_str().expect("texture value was not string");
        let first_char = val.chars().nth(0).expect("texture string had length of 0");
        if val.len() == 1 {
            panic!("invalid texture variable length")
        }
        if first_char == '#' {
            return InternedTextureVariable::Variable(
                rodeo.get_or_intern(val.strip_prefix('#').unwrap()),
            );
        } else {
            return InternedTextureVariable::ResourcePath(rodeo.get_or_intern(val));
        }
    }
}
