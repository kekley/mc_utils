use fxhash::FxHashMap;
use glam::Vec4;
use hashbrown::HashMap;
use serde_json::Value;
use smol_str::SmolStr;

use super::{block_models::ASSET_PATH, utils::parse_vec4};

#[derive(Debug)]
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
#[derive(Debug, Hash, PartialEq, Eq)]
pub enum TextureVariable {
    Variable(SmolStr),
    ResourcePath(SmolStr),
}

impl TextureVariable {
    pub fn get(&self) -> &SmolStr {
        match self {
            TextureVariable::Variable(smol_str) => smol_str,
            TextureVariable::ResourcePath(smol_str) => smol_str,
        }
    }
}
#[derive(Debug)]
pub struct BlockTextures {
    pub textures: FxHashMap<SmolStr, TextureVariable>,
}

impl BlockTextures {
    pub fn get_all(&self) -> HashMap<SmolStr, SmolStr> {
        self.textures
            .iter()
            .map(|entry| (entry.0.to_owned(), Self::path_inner(entry.1)))
            .collect()
    }
    pub fn get_keys(&self) -> Vec<SmolStr> {
        self.textures
            .keys()
            .into_iter()
            .map(|key| key.clone())
            .collect()
    }

    fn path_inner(variable: &TextureVariable) -> SmolStr {
        let path_str = variable.get();
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

    pub fn parse_textures(value: &Value) -> Self {
        let obj = value.as_object().expect("textures was not object");
        let textures = obj
            .iter()
            .map(|(var1, var2)| {
                let name = SmolStr::new(var1);
                let texture = TextureVariable::from(var2);
                (name, texture)
            })
            .collect::<FxHashMap<SmolStr, TextureVariable>>();
        BlockTextures { textures }
    }
}
impl From<&Value> for TextureVariable {
    fn from(value: &Value) -> Self {
        let val = value.as_str().expect("texture value was not SmolStr");
        let first_char = val.chars().nth(0).expect("texture SmolStr had length of 0");
        if val.len() == 1 {
            panic!("invalid texture variable length")
        }
        if first_char == '#' {
            return TextureVariable::Variable(SmolStr::new(val));
        } else {
            return TextureVariable::ResourcePath(SmolStr::new(val));
        }
    }
}

impl From<&str> for TextureVariable {
    fn from(value: &str) -> Self {
        let first_char = value
            .chars()
            .nth(0)
            .expect("texture SmolStr had length of 0");
        if value.len() == 1 {
            panic!("invalid texture variable length")
        }
        if first_char == '#' {
            return TextureVariable::Variable(SmolStr::new(value));
        } else {
            return TextureVariable::ResourcePath(SmolStr::new(value));
        }
    }
}
