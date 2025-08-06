use serde_json::{Map, Value};
use smol_str::SmolStr;

use super::{
    block_models::ASSET_PATH,
    resource_error::ResourceErrorKind,
    utils::{parse_array, parse_type},
};

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
    type Error = ResourceErrorKind;
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
#[derive(Debug, Clone, Hash, PartialEq, Eq)]

pub struct TextureVariable {
    pub value: String,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub struct TexturePath {
    pub value: String,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TextureVariableEnum {
    Variable(TextureVariable),
    ResourcePath(TexturePath),
}

#[derive(Debug, Clone)]
pub struct BlockTextureMap {
    pub texture_variables: Vec<(TextureVariable, TextureVariableEnum)>,
}

impl BlockTextureMap {
    pub fn combine(&mut self, other: BlockTextureMap) {
        for texture in &other.texture_variables {
            if !self.texture_variables.contains(texture) {
                self.texture_variables.push(texture.clone());
            }
        }
    }
    pub fn as_slice(&self) -> &[(TextureVariable, TextureVariableEnum)] {
        self.texture_variables.as_slice()
    }

    fn to_path(variable: &TextureVariableEnum) -> SmolStr {
        let inner_str = match variable {
            TextureVariableEnum::Variable(_texture_variable) => {
                panic!("cannot resolve a texture variable to a path")
            }
            TextureVariableEnum::ResourcePath(texture_path) => texture_path.value.as_str(),
        };

        let (namespace, remaining_str) = inner_str.split_once(":").unwrap_or(("", inner_str));

        let (texture_type, remaining_str) = remaining_str
            .split_once("/")
            .expect("invalid path for texture");
        let mut path = ASSET_PATH.to_string();
        path.push_str(namespace);
        path.push('/');
        path.push_str("textures/");
        path.push_str(texture_type);
        path.push('/');
        path.push_str(remaining_str);
        path.push_str(".png");

        SmolStr::from(path)
    }

    pub fn try_from_json(value: &Value) -> Result<Self, ResourceErrorKind> {
        let json_object = parse_type::<Map<_, _>>(value)?;
        let a = json_object
            .iter()
            .map(|(var1, var2)| {
                let var = TextureVariable {
                    value: String::from(var1),
                };

                let texture = TextureVariableEnum::try_from(var2)?;
                Ok((var, texture))
            })
            .collect::<Result<Vec<_>, ResourceErrorKind>>()?;

        Ok(BlockTextureMap {
            texture_variables: a,
        })
    }
}

impl TextureVariableEnum {
    pub(crate) fn try_from(value: &Value) -> Result<Self, ResourceErrorKind> {
        let val = parse_type::<&str>(value)?;

        let first_char = val.chars().nth(0).ok_or(ResourceErrorKind::InvalidField(
            "Empty str for texture".to_string(),
        ))?;

        if first_char == '#' {
            Ok(TextureVariableEnum::Variable(TextureVariable {
                value: String::from(val.strip_prefix('#').unwrap()),
            }))
        } else {
            Ok(TextureVariableEnum::ResourcePath(TexturePath {
                value: String::from(val),
            }))
        }
    }
}
