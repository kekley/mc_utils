use std::sync::Arc;

use bumpalo::Bump;
use serde_json::Value;

use super::{
    block_models::BlockRotation,
    block_texture::{TextureVariableEnum, Uv},
    resource_error::ResourceErrorKind,
};

#[derive(Debug, Clone, Copy)]
pub struct TintIndex(i64);

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
        bump: &'a mut Bump,
    ) -> Result<[Option<BlockFace<'a>>; 6], ResourceErrorKind> {
        const NONE_VALUE: Option<BlockFace> = None;
        let vec = value
            .as_object()
            .context("\"faces\" field was not a json object")?
            .iter()
            .enumerate()
            .map(|(i, (name, value))| BlockFace::parse_from_json_value(name, value, bump))
            .collect::<Result<bumpalo::collections::Vec<'a, _>, ResourceErrorKind>>()?;

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
            _ => Err(format!("Invalid value {value} for \"facename\"")),
        }
    }
}

impl<'a> BlockFace<'a> {
    pub fn parse_from_json_value(
        face_name: &str,
        value: &Value,
        bump: &'a mut Bump,
    ) -> Result<Self, ResourceErrorKind> {
        let name = FaceName::try_from(face_name)?;
        let uv = value
            .get("uv")
            .map(|value| Uv::try_from(value))
            .transpose()?;
        let texture = TextureVariableEnum::parse_from_json_value(
            value.get("texture").expect("no texture for face"),
            bump,
        );
        let cullface = value
            .get("cullface")
            .map(|value| {
                FaceName::try_from(
                    value
                        .as_str()
                        .context("\"cullface\" field was not a string")?,
                )
            })
            .transpose()?;
        let rotation = value
            .get("rotation")
            .map(|value| BlockRotation::try_from(value))
            .transpose()?;
        let tint = value
            .get("tint")
            .map(|value| value.as_i64().context("\"tint\" field was not an integer"))
            .transpose()?
            .map(|ind| TintIndex(ind));

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
