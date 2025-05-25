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

use crate::utils::parse_type;

use super::{
    block_models::BlockRotation,
    block_texture::{TextureVariableEnum, Uv},
    resource_error::ResourceErrorKind,
    utils::{get_optional_field, try_get_field},
};

#[derive(Debug, Clone, Copy)]
pub struct TintIndex(i32);

impl From<i32> for TintIndex {
    fn from(value: i32) -> Self {
        TintIndex(value)
    }
}

impl TryFrom<&Value> for TintIndex {
    type Error = ResourceErrorKind;

    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let ind = parse_type::<i32>(value)?;
        Ok(ind.into())
    }
}

#[derive(Debug, Clone)]
pub struct BlockFace<'a> {
    pub name: FaceName,
    pub uv: Option<Uv>,
    pub texture: TextureVariableEnum<'a>,
    cullface: Option<FaceName>,
    pub texture_rotation: Option<BlockRotation>,
    pub tint_index: Option<TintIndex>,
}

impl<'a> BlockFace<'a> {
    pub fn parse_faces(
        value: &Value,
        bump: &'a Bump,
    ) -> Result<[Option<BlockFace<'a>>; 6], ResourceErrorKind> {
        const NONE_VALUE: Option<BlockFace> = None;
        let obj = parse_type::<Map<_, _>>(value)?;
        let vec = obj
            .iter()
            .map(|(name, value)| BlockFace::parse_from_json_value(name, value, bump))
            .collect_in::<Result<BumpVec<'a, _>, ResourceErrorKind>>(bump)?;

        let mut array = [NONE_VALUE; 6];
        vec.iter()
            .for_each(|face| array[face.name as usize] = Some(face.clone()));
        Ok(array)
    }
}

#[derive(Debug, Clone, Copy)]
pub enum FaceName {
    Down = 2,
    Up = 3,
    North = 4,
    South = 5,
    West = 0,
    East = 1,
}
impl TryFrom<&str> for FaceName {
    type Error = ResourceErrorKind;
    fn try_from(value: &str) -> Result<Self, Self::Error> {
        match value {
            "down" => Ok(FaceName::Down),
            "up" => Ok(FaceName::Up),
            "north" => Ok(FaceName::North),
            "south" => Ok(FaceName::South),
            "west" => Ok(FaceName::West),
            "east" => Ok(FaceName::East),
            _ => Err(ResourceErrorKind::InvalidField(format!(
                "Invalid value {value} for \"facename\""
            ))),
        }
    }
}

impl<'a> BlockFace<'a> {
    pub fn parse_from_json_value(
        face_name: &str,
        value: &Value,
        bump: &'a Bump,
    ) -> Result<Self, ResourceErrorKind> {
        let name = FaceName::try_from(face_name)?;
        let uv_field = get_optional_field(value, "uv");
        let uv = match uv_field {
            Some(value) => Some(Uv::try_from(value)?),
            None => None,
        };

        let texture_field = try_get_field(value, "texture")?;
        let texture = TextureVariableEnum::try_from_in(texture_field, bump)?;
        let cullface_field = get_optional_field(value, "cullface");

        let cullface = match cullface_field {
            Some(value) => Some(match parse_type::<&str>(value) {
                Ok(str) => FaceName::try_from(str)?,
                Err(err) => {
                    return Err(err);
                }
            }),
            None => None,
        };

        let rotation_field = get_optional_field(value, "rotation");
        let rotation = match rotation_field {
            Some(value) => Some(BlockRotation::try_from(value)?),
            None => None,
        };
        let tint_field = get_optional_field(value, "tint");
        let tint = match tint_field {
            Some(value) => Some(TintIndex::try_from(value)?),
            None => None,
        };

        Ok(BlockFace {
            name,
            uv,
            texture,
            cullface,
            texture_rotation: rotation,
            tint_index: tint,
        })
    }
}
