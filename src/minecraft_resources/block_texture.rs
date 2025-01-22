use fxhash::FxHashMap;
use serde_json::Value;

use super::utils::parse_vec4;


#[derive(Debug)]
pub struct Uv {
    x1: f32,
    y1: f32,
    x2: f32,
    y2: f32,
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
    Variable(String),
    ResourcePath(String),
}
#[derive(Debug)]
pub struct BlockTextures {
    textures: FxHashMap<TextureVariable, TextureVariable>,
}

impl BlockTextures {
    pub fn parse_textures(value: &Value) -> Self {
        let obj = value.as_object().expect("textures was not object");
        let textures = obj
            .iter()
            .map(|(var1, var2)| {
                let name = TextureVariable::from(var1.as_str());
                let texture = TextureVariable::from(var2);
                (name, texture)
            })
            .collect::<FxHashMap<TextureVariable, TextureVariable>>();
        todo!()
    }
}
impl From<&Value> for TextureVariable {
    fn from(value: &Value) -> Self {
        let string_val = value.as_str().expect("texture value was not string");
        let first_char = string_val
            .chars()
            .nth(0)
            .expect("texture string had length of 0");
        if string_val.len() == 1 {
            panic!("invalid texture variable length")
        }
        if first_char == '#' {
            return TextureVariable::Variable(string_val.to_string());
        } else {
            return TextureVariable::ResourcePath(string_val.to_string());
        }
    }
}

impl From<&str> for TextureVariable {
    fn from(value: &str) -> Self {
        let first_char = value
            .chars()
            .nth(0)
            .expect("texture string had length of 0");
        if value.len() == 1 {
            panic!("invalid texture variable length")
        }
        if first_char == '#' {
            return TextureVariable::Variable(value.to_string());
        } else {
            return TextureVariable::ResourcePath(value.to_string());
        }
    }
}
