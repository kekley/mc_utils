#![warn(
    clippy::all,
    clippy::restriction,
    clippy::pedantic,
    clippy::nursery,
    clippy::cargo
)]
use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::{collections::CollectIn, Bump};
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

struct TextureVariable<'a> {
    pub value: BumpString<'a>,
}
#[derive(Debug, Clone, Hash, PartialEq, Eq)]
struct TexturePath<'a> {
    pub value: BumpString<'a>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum TextureVariableEnum<'a> {
    Variable(TextureVariable<'a>),
    ResourcePath(TexturePath<'a>),
}

#[derive(Debug, Clone)]
pub struct BlockTextureMap<'a> {
    pub texture_variables: BumpVec<'a, (TextureVariable<'a>, TextureVariableEnum<'a>)>,
}

impl<'a> BlockTextureMap<'a> {
    pub fn combine(&mut self, other: BlockTextureMap<'a>) {
        for texture in &other.texture_variables {
            if !self.texture_variables.contains(&texture) {
                self.texture_variables.push(texture.clone());
            }
        }
    }
    pub fn as_slice(&self) -> &[(TextureVariable, TextureVariableEnum)] {
        self.texture_variables.as_slice()
    }

    fn to_path(variable: &TextureVariableEnum) -> SmolStr {
        let inner_str = match variable {
            TextureVariableEnum::Variable(texture_variable) => {
                panic!("cannot resolve a texture variable to a path")
            }
            TextureVariableEnum::ResourcePath(texture_path) => texture_path.value.as_str(),
        };

        let (namespace, remaining_str) = inner_str.split_once(":").unwrap_or(("", inner_str));

        let (texture_type, remaining_str) = remaining_str
            .split_once("/")
            .expect("invalid path for texture");
        SmolStr::from(
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

    pub fn try_from_json(value: &Value, bump: &'a Bump) -> Result<Self, ResourceErrorKind> {
        let json_object = parse_type::<Map<_, _>>(value)?;
        let a = json_object
            .iter()
            .map(|(var1, var2)| {
                let var = TextureVariable {
                    value: BumpString::from_str_in(&var1, bump),
                };

                let texture = TextureVariableEnum::try_from_in(var2, bump)?;
                Ok((var, texture))
            })
            .collect_in::<Result<BumpVec<'a, _>, ResourceErrorKind>>(bump)?;

        Ok(BlockTextureMap {
            texture_variables: a,
        })
    }
}

impl<'a> TextureVariableEnum<'a> {
    pub(crate) fn try_from_in(value: &Value, bump: &'a Bump) -> Result<Self, ResourceErrorKind> {
        let val = parse_type::<&str>(value)?;

        let first_char = val
            .chars()
            .nth(0)
            .ok_or(ResourceErrorKind::InvalidField(format!(
                "Empty str for texture"
            )))?;

        if first_char == '#' {
            return Ok(TextureVariableEnum::Variable(TextureVariable {
                value: BumpString::from_str_in(val.strip_prefix('#').unwrap(), bump),
            }));
        } else {
            return Ok(TextureVariableEnum::ResourcePath(TexturePath {
                value: BumpString::from_str_in(val, bump),
            }));
        }
    }
}
