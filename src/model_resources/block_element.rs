use std::sync::Arc;

use glam::{Affine3A, Mat3A, Mat4, Quat, Vec2, Vec3, Vec3A};
use lasso::ThreadedRodeo;
use serde_json::Value;

use super::{block_face::InternedFace, utils::parse_vec3};
pub type Shade = bool;
#[derive(Debug, Clone)]
pub struct InternedBlockElement {
    pub from: Vec3,
    pub to: Vec3,
    pub rotation: Option<ElementRotation>,
    shade: Option<Shade>,
    pub faces: [Option<InternedFace>; 6],
}

impl InternedBlockElement {
    pub fn is_cube(&self) -> bool {
        !self.faces.iter().any(|f| f.is_none())
            && self.rotation.is_none()
            && self.from == Vec3::ZERO
            && self.to == Vec3::splat(16.0)
    }
    pub fn is_axis_aligned(&self) -> bool {
        match &self.rotation {
            Some(rotation) => rotation.angle == 0.0,
            None => true,
        }
    }
    pub fn from_json_value(value: &Value, rodeo: &Arc<ThreadedRodeo>) -> Self {
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
        let faces =
            InternedFace::parse_faces(value.get("faces").expect("faces not defined"), rodeo);
        InternedBlockElement {
            from,
            to,
            rotation,
            shade,
            faces,
        }
    }
    pub fn parse_elements(value: &Value, rodeo: &Arc<ThreadedRodeo>) -> Vec<InternedBlockElement> {
        value
            .as_array()
            .expect("elements was not an array")
            .iter()
            .map(|value| InternedBlockElement::from_json_value(value, rodeo))
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

#[derive(Debug, Clone, Copy)]
enum ElementAxis {
    X,
    Y,
    Z,
}
pub type Angle = f32;
pub type Rescale = bool;
#[derive(Debug, Clone)]
pub struct ElementRotation {
    origin: Vec3,
    axis: ElementAxis,
    angle: Angle,
    rescale: Option<Rescale>,
}

impl ElementRotation {
    pub fn to_matrix(&self) -> Affine3A {
        let origin = self.origin;
        let radians = self.angle.to_radians();
        let rotation_axis = match self.axis {
            ElementAxis::X => Vec3::X,
            ElementAxis::Y => Vec3::Y,
            ElementAxis::Z => Vec3::Z,
        };
        let matrix = Affine3A::from_scale_rotation_translation(
            Vec3::ONE,
            Quat::from_axis_angle(rotation_axis, radians),
            -origin,
        );

        if self.rescale.unwrap_or(false) {}
        matrix
    }
}
