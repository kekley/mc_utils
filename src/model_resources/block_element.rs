use bumpalo::collections::String as BumpString;
use bumpalo::collections::Vec as BumpVec;
use bumpalo::{collections::CollectIn, Bump};
use serde_json::Value;

use super::{
    block_face::BlockFace,
    resource_error::ResourceErrorKind,
    utils::{get_optional_field, parse_array, parse_type, try_get_field},
};

#[derive(Debug, Clone, Copy)]
pub struct Shade(bool);
impl From<bool> for Shade {
    fn from(value: bool) -> Self {
        Self(value)
    }
}
impl From<&bool> for Shade {
    fn from(value: &bool) -> Self {
        Self(*value)
    }
}

#[derive(Debug, Clone)]
pub struct BlockElement<'a> {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub rotation: Option<ElementRotation>,
    shade: Option<Shade>,
    pub faces: [Option<BlockFace<'a>>; 6],
}

impl<'a> BlockElement<'a> {
    pub fn is_cube(&self) -> bool {
        !self.faces.iter().any(|f| f.is_none())
            && self.rotation.is_none()
            && self.from == [0f32; 3]
            && self.to == [16.0f32; 3]
    }
    pub fn is_axis_aligned(&self) -> bool {
        match &self.rotation {
            Some(rotation) => rotation.angle.0 == 0.0,
            None => true,
        }
    }
    pub fn from_json_value(value: &Value, bump: &'a Bump) -> Result<Self, ResourceErrorKind> {
        let from = parse_array::<f32, 3>(try_get_field(value, "from")?)?;
        let to = parse_array::<f32, 3>(try_get_field(value, "to")?)?;
        let rotation_field = get_optional_field(value, "rotation");

        let rotation = match rotation_field {
            Some(field) => Some(ElementRotation::try_from(field)?),
            None => None,
        };

        let shade_field = get_optional_field(value, "shade");

        let shade = match shade_field {
            Some(field) => Some(parse_type::<bool>(field)?.into()),
            None => None,
        };

        let faces = BlockFace::parse_faces(try_get_field(value, "faces")?, bump)?;

        Ok(BlockElement {
            from,
            to,
            rotation,
            shade,
            faces,
        })
    }
    pub fn parse_elements(
        value: &Value,
        bump: &'a Bump,
    ) -> Result<BumpVec<'a, BlockElement<'a>>, ResourceErrorKind> {
        let vec = parse_type::<Vec<_>>(value)?;
        let iter = vec
            .iter()
            .map(|value| BlockElement::from_json_value(value, bump));
        let a = iter.collect_in(bump);
        a
    }
}

impl TryFrom<&Value> for ElementAxis {
    type Error = ResourceErrorKind;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        match value.as_str() {
            Some(str) => match str {
                "x" => Ok(ElementAxis::X),
                "y" => Ok(ElementAxis::Y),
                "z" => Ok(ElementAxis::Z),
                "X" => Ok(ElementAxis::X),
                "Y" => Ok(ElementAxis::Y),
                "Z" => Ok(ElementAxis::Z),
                _ => Err(ResourceErrorKind::InvalidField(format!(
                    "Expected x,y, or z for element axis, got {}",
                    str
                ))),
            },
            None => {
                return Err(ResourceErrorKind::InvalidField(format!(
                    "Wrong data type for element axis. Value:{}",
                    value.to_string()
                )))
            }
        }
    }
}

impl TryFrom<&Value> for ElementRotation {
    type Error = ResourceErrorKind;
    fn try_from(value: &Value) -> Result<Self, Self::Error> {
        let origin_field = try_get_field(value, "origin")?;
        let origin = parse_array::<f32, 3>(origin_field)?;
        let axis_field = try_get_field(value, "axis")?;
        let axis = ElementAxis::try_from(axis_field)?;
        let angle_field = try_get_field(value, "angle")?;
        let angle = parse_type::<f32>(angle_field)?;
        let rescale_field = get_optional_field(value, "rescale");
        let rescale = match rescale_field {
            Some(field) => Some(parse_type::<bool>(field)?.into()),
            None => None,
        };
        Ok(ElementRotation {
            origin,
            axis,
            angle: angle.try_into()?,
            rescale: rescale,
        })
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ElementAxis {
    X,
    Y,
    Z,
}
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Angle(f32);

impl TryFrom<f32> for Angle {
    type Error = ResourceErrorKind;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        match value.is_finite() {
            true => Ok(Angle(value)),
            false => Err(ResourceErrorKind::InvalidField(format!(
                "Infinite or NaN angle value"
            ))),
        }
    }
}
impl TryFrom<&f32> for Angle {
    type Error = ResourceErrorKind;

    fn try_from(value: &f32) -> Result<Self, Self::Error> {
        match value.is_finite() {
            true => Ok(Angle(*value)),
            false => Err(ResourceErrorKind::InvalidField(format!(
                "Infinite or NaN angle value"
            ))),
        }
    }
}
impl TryFrom<f64> for Angle {
    type Error = ResourceErrorKind;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        match value.is_finite() {
            true => Ok(Angle(value as f32)),
            false => Err(ResourceErrorKind::InvalidField(format!(
                "Infinite or NaN angle value"
            ))),
        }
    }
}
impl TryFrom<&f64> for Angle {
    type Error = ResourceErrorKind;

    fn try_from(value: &f64) -> Result<Self, Self::Error> {
        match value.is_finite() {
            true => Ok(Angle(*value as f32)),
            false => Err(ResourceErrorKind::InvalidField(format!(
                "Infinite or NaN angle value"
            ))),
        }
    }
}
impl Into<f32> for Angle {
    fn into(self) -> f32 {
        self.0
    }
}

impl<'a> Into<&'a f32> for &'a Angle {
    fn into(self) -> &'a f32 {
        &self.0
    }
}
#[derive(Debug, Clone, Copy)]
pub struct Rescale(bool);
impl From<bool> for Rescale {
    fn from(value: bool) -> Self {
        Self(value)
    }
}
impl From<&bool> for Rescale {
    fn from(value: &bool) -> Self {
        Self(*value)
    }
}

#[derive(Debug, Clone)]
pub struct ElementRotation {
    pub origin: [f32; 3],
    pub axis: ElementAxis,
    pub angle: Angle,
    pub rescale: Option<Rescale>,
}
