use std::sync::Arc;

use anyhow::{anyhow, Context, Error};
use lasso::ThreadedRodeo;
use serde_json::Value;

use super::{block_face::InternedFace, utils::parse_array};

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
pub struct InternedBlockElement {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub rotation: Option<ElementRotation>,
    shade: Option<Shade>,
    pub faces: [Option<InternedFace>; 6],
}

impl InternedBlockElement {
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
    pub fn from_json_value(value: &Value, rodeo: &Arc<ThreadedRodeo>) -> anyhow::Result<Self> {
        let from = parse_array::<f32, 3>(
            value
                .get("from")
                .context("\"from\" field did not exist in block element")?,
        )?;
        let to = parse_array::<f32, 3>(
            value
                .get("to")
                .expect("\"to\" field did not exist for block element"),
        )?;
        let rotation: Option<ElementRotation> = value
            .get("rotation")
            .map(|value| ElementRotation::try_from(value))
            .transpose()?;
        let shade: Option<Shade> = value
            .get("shade")
            .map(|value| value.as_bool().context("shade existed but was not bool"))
            .transpose()?
            .map(|bool| bool.into());

        let faces: [Option<InternedFace>; 6] = InternedFace::parse_faces(
            value
                .get("faces")
                .context("\"faces\" field not defined for block element")?,
            rodeo,
        )?;
        Ok(InternedBlockElement {
            from,
            to,
            rotation,
            shade,
            faces,
        })
    }
    pub fn parse_elements(
        value: &Value,
        rodeo: &Arc<ThreadedRodeo>,
    ) -> anyhow::Result<Vec<InternedBlockElement>> {
        value
            .as_array()
            .context("Attempted to parse block elements that were not in an array")?
            .iter()
            .map(|value| InternedBlockElement::from_json_value(value, rodeo))
            .collect::<anyhow::Result<Vec<_>>>()
    }
}

impl TryFrom<&Value> for ElementAxis {
    type Error = anyhow::Error;
    fn try_from(value: &Value) -> Result<Self, Error> {
        match value.as_str().context(format!(
            "Element axis value must be a string, json value : {}",
            value.to_string()
        ))? {
            "x" => Ok(ElementAxis::X),
            "y" => Ok(ElementAxis::Y),
            "z" => Ok(ElementAxis::Z),
            "X" => Ok(ElementAxis::X),
            "Y" => Ok(ElementAxis::Y),
            "Z" => Ok(ElementAxis::Z),
            _ => Err(anyhow!("Element axis value must be x, y or z")),
        }
    }
}

impl TryFrom<&Value> for ElementRotation {
    type Error = anyhow::Error;
    fn try_from(value: &Value) -> Result<Self, Error> {
        let origin = parse_array::<f32, 3>(
            value
                .get("origin")
                .context("Element rotation was missing \"origin\" field ")?,
        )?;
        let axis = ElementAxis::try_from(
            value
                .get("axis")
                .context("Element rotation was missing \"axis\" field")?,
        )?;
        let angle = value
            .get("angle")
            .context("Element rotation was missing \"angle\" field")?
            .as_f64()
            .context("\"angle\"field was not a number")? as f32;
        let rescale = value.get("rescale");
        let rescale: Option<Rescale> = match rescale {
            Some(value) => Some(
                value
                    .as_bool()
                    .context("\"rescale\" value was not a boolean")?
                    .into(),
            ),
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
    type Error = anyhow::Error;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        match value.is_finite() {
            true => Ok(Angle(value)),
            false => Err(anyhow!("Infinite or NaN angle value")),
        }
    }
}
impl TryFrom<&f32> for Angle {
    type Error = anyhow::Error;

    fn try_from(value: &f32) -> Result<Self, Self::Error> {
        match value.is_finite() {
            true => Ok(Angle(*value)),
            false => Err(anyhow!("Infinite or NaN angle value")),
        }
    }
}
impl TryFrom<f64> for Angle {
    type Error = anyhow::Error;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        match value.is_finite() {
            true => Ok(Angle(value as f32)),
            false => Err(anyhow!("Infinite or NaN angle value")),
        }
    }
}
impl TryFrom<&f64> for Angle {
    type Error = anyhow::Error;

    fn try_from(value: &f64) -> Result<Self, Self::Error> {
        match value.is_finite() {
            true => Ok(Angle(*value as f32)),
            false => Err(anyhow!("Infinite or NaN angle value")),
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
