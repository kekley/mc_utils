use std::sync::Arc;

use anyhow::{anyhow, Context, Error, Result};
use lasso::ThreadedRodeo;
use log::error;
use serde_json::Value;

use super::{
    block_models::BlockRotation,
    block_texture::{InternedTextureVariable, Uv},
};

#[derive(Debug, Clone, Copy)]
pub struct TintIndex(i64);

#[derive(Debug, Clone)]
pub struct InternedFace {
    pub name: FaceName,
    pub uv: Option<Uv>,
    pub texture: InternedTextureVariable,
    cullface: Option<FaceName>,
    pub texture_rotation: Option<BlockRotation>,
    pub tint_index: Option<TintIndex>,
}

impl InternedFace {
    pub fn parse_faces(
        value: &Value,
        rodeo: &Arc<ThreadedRodeo>,
    ) -> anyhow::Result<[Option<InternedFace>; 6]> {
        const NONE_VALUE: Option<InternedFace> = None;
        let vec = value
            .as_object()
            .context("\"faces\" field was not a json object")?
            .iter()
            .enumerate()
            .map(|(i, (name, value))| InternedFace::parse_from_json_value(name, value, rodeo))
            .collect::<Result<Vec<_>>>()?;

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
    type Error = anyhow::Error;
    fn try_from(value: &str) -> Result<Self, Error> {
        match value {
            "down" => Ok(FaceName::Down),
            "up" => Ok(FaceName::Up),
            "north" => Ok(FaceName::North),
            "south" => Ok(FaceName::South),
            "west" => Ok(FaceName::West),
            "east" => Ok(FaceName::East),
            _ => Err(anyhow!(format!("Invalid value {value} for \"facename\""))),
        }
    }
}

impl InternedFace {
    pub fn parse_from_json_value(
        face_name: &str,
        value: &Value,
        rodeo: &Arc<ThreadedRodeo>,
    ) -> anyhow::Result<Self> {
        let name = FaceName::try_from(face_name)?;
        let uv = value
            .get("uv")
            .map(|value| Uv::try_from(value))
            .transpose()?;
        let texture = InternedTextureVariable::parse_from_json_value(
            value.get("texture").expect("no texture for face"),
            rodeo,
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

        Ok(InternedFace {
            name,
            uv,
            texture,
            cullface,
            texture_rotation: rotation,
            tint_index: tint,
        })
    }
}
