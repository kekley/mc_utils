use glam::Vec3;
use serde_json::Value;

use super::{block_face::Face, utils::parse_vec3};

#[derive(Debug)]
pub struct BlockElement {
    from: Vec3,
    to: Vec3,
    rotation: Option<ElementRotation>,
    shade: Option<bool>,
    faces: [Option<Face>; 6],
}

impl BlockElement {
    pub fn parse_elements(value: &Value) -> Vec<BlockElement> {
        value
            .as_array()
            .expect("elements was not an array")
            .iter()
            .map(|value| BlockElement::from(value))
            .collect::<Vec<_>>()
    }
    
}

impl From<&Value> for ElementAxis {
    fn from(value: &Value) -> Self {
        match value.as_str().expect("axis was not str") {
            "x" => ElementAxis::X,
            "y" => ElementAxis::Y,
            "z" => ElementAxis::Z,
            _ => {
                panic!("invalid axis for element")
            }
        }
    }
}

impl From<&Value> for ElementRotation {
    fn from(value: &Value) -> Self {
        let origin = parse_vec3(value.get("origin").expect("rotation missing origin"));
        let axis = ElementAxis::from(value.get("axis").expect("rotation missing axis"));
        let angle = value
            .get("angle")
            .expect("rotation missing angle")
            .as_f64()
            .expect("angle was not number") as f32;
        let rescale = value
            .get("rescale")
            .map(|value| value.as_bool().expect("rescale was not bool"));
        ElementRotation {
            origin,
            axis,
            angle,
            rescale,
        }
    }
}

impl From<&Value> for BlockElement {
    fn from(value: &Value) -> Self {
        let from = parse_vec3(
            value
                .get("from")
                .expect("from does not exist in block element"),
        );
        let to = parse_vec3(value.get("to").expect("to does not exist in block element"));
        let rotation = value
            .get("rotation")
            .map(|value| ElementRotation::from(value));
        let shade = value
            .get("shade")
            .map(|value| value.as_bool().expect("shade existed but was not bool"));
        let faces = Face::parse_faces(value.get("faces").expect("faces not defined"));
        BlockElement {
            from,
            to,
            rotation,
            shade,
            faces,
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum ElementAxis {
    X,
    Y,
    Z,
}
#[derive(Debug)]
pub struct ElementRotation {
    origin: Vec3,
    axis: ElementAxis,
    angle: f32,
    rescale: Option<bool>,
}
