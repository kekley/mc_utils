use glam::{Mat3A, Vec2, Vec3, Vec3A};
use serde_json::Value;

use super::{block_face::Face, utils::parse_vec3};
pub type Shade = bool;
#[derive(Debug)]
pub struct BlockElement {
    pub from: Vec3,
    pub to: Vec3,
    pub rotation: Option<ElementRotation>,
    shade: Option<Shade>,
    pub faces: [Option<Face>; 6],
}

impl BlockElement {
    pub fn is_cube(&self) -> bool {
        !self.faces.iter().any(|f| f.is_none())
    }
    pub fn is_axis_aligned(&self) -> bool {
        match &self.rotation {
            Some(rotation) => rotation.angle == 0.0,
            None => true,
        }
    }
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
pub type Angle = f32;
pub type Rescale = bool;
#[derive(Debug)]
pub struct ElementRotation {
    origin: Vec3,
    axis: ElementAxis,
    angle: Angle,
    rescale: Option<Rescale>,
}

impl ElementRotation {
    pub fn to_matrix(&self) -> Mat3A {
        let radians = self.angle.to_radians();

        let rotation = match self.axis {
            ElementAxis::X => Mat3A::from_rotation_x(radians),
            ElementAxis::Y => Mat3A::from_rotation_y(radians),
            ElementAxis::Z => Mat3A::from_rotation_z(radians),
        };

        if self.rescale.unwrap_or(false) {
            return Mat3A::from_scale(Vec2::splat(1.0)) * rotation;
        }
        rotation
    }
}
