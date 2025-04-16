use std::sync::Arc;

use lasso::ThreadedRodeo;
use serde_json::Value;

use super::{
    block_models::BlockRotation,
    block_texture::{TextureVariable, Uv},
    utils::parse_vec4,
};
pub type TintIndex = i64;

#[derive(Debug, Clone)]
pub struct Face {
    pub name: FaceName,
    pub uv: Option<Uv>,
    pub texture: TextureVariable,
    cullface: Option<FaceName>,
    pub texture_rotation: Option<BlockRotation>,
    pub tint_index: Option<TintIndex>,
}

impl Face {
    pub fn parse_faces(value: &Value, rodeo: &Arc<ThreadedRodeo>) -> [Option<Face>; 6] {
        const NONE_VALUE: Option<Face> = None;
        let mut face_array = [NONE_VALUE; 6];
        value
            .as_object()
            .expect("faces was not object")
            .iter()
            .enumerate()
            .for_each(|(i, (name, value))| {
                face_array[i] = Some(Face::parse_from_json_value(name, value, rodeo))
            });

        face_array
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
impl From<&str> for FaceName {
    fn from(value: &str) -> Self {
        match value {
            "down" => FaceName::Down,
            "up" => FaceName::Up,
            "north" => FaceName::North,
            "south" => FaceName::South,
            "west" => FaceName::West,
            "east" => FaceName::East,
            _ => panic!("invalid value for facename"),
        }
    }
}

impl Face {
    pub fn parse_from_json_value(
        face_name: &str,
        value: &Value,
        rodeo: &Arc<ThreadedRodeo>,
    ) -> Self {
        let name = FaceName::from(face_name);
        let value = value;
        let uv = value
            .get("uv")
            .map(|value| Uv::from(parse_vec4(value).to_array()));
        let texture = TextureVariable::parse_from_json_value(
            value.get("texture").expect("no texture for face"),
            rodeo,
        );
        let cullface = value
            .get("cullface")
            .map(|value| FaceName::from(value.as_str().expect("cullface was not str")));
        let rotation = value
            .get("rotation")
            .map(|value| BlockRotation::from(value));
        let tint = value
            .get("tint")
            .map(|value| value.as_i64().expect("tint was not integer"));

        Face {
            name,
            uv,
            texture,
            cullface,
            texture_rotation: rotation,
            tint_index: tint,
        }
    }
}
