use serde_json::Value;

use super::utils::{parse_array, parse_f32_3};

#[derive(Debug, Clone)]
enum DisplayPosition {
    ThirdPersonRightHand,
    ThirdPersonLeftHand,
    FirstPersonRightHand,
    FirstPersonLeftHand,
    Gui,
    Head,
    Ground,
    Fixed,
}

impl From<&str> for DisplayPosition {
    fn from(value: &str) -> DisplayPosition {
        match value {
            "gui" => DisplayPosition::Gui,
            "ground" => DisplayPosition::Ground,
            "fixed" => DisplayPosition::Fixed,
            "thirdperson_righthand" => DisplayPosition::ThirdPersonRightHand,
            "thirdperson_lefthand" => DisplayPosition::ThirdPersonLeftHand,
            "firstperson_righthand" => DisplayPosition::FirstPersonRightHand,
            "firstperson_lefthand" => DisplayPosition::FirstPersonLeftHand,
            "head" => DisplayPosition::Head,
            _ => panic!("invalid display enum"),
        }
    }
}
#[derive(Debug, Clone)]
pub struct BlockDisplay {
    position: DisplayPosition,
    rotation: [f32; 3],
    translation: [f32; 3],
    scale: [f32; 3],
}

impl TryFrom<(&str, &Value)> for BlockDisplay {
    type Error = anyhow::Error;

    fn try_from(value: (&str, &Value)) -> anyhow::Result<Self> {
        let position = value.0;
        let data = value.1;
        let rotation = data.get("rotation").map(|f| parse_array::<f64, 3>(value))?;
        let translation = data
            .get("translation")
            .map(|f| parse_f32_3(f))
            .unwrap_or(Vec3::ZERO);
        let scale = data
            .get("scale")
            .map(|f| parse_f32_3(f))
            .unwrap_or(Vec3::ZERO);
        let res = BlockDisplay {
            position: position.try_into()?,
            rotation,
            translation,
            scale,
        };

        Ok(res)
    }
}

impl BlockDisplay {
    pub fn parse_display(value: &Value) -> Vec<BlockDisplay> {
        value
            .as_object()
            .expect("display was not object")
            .iter()
            .map(|(name, value)| BlockDisplay::try_from((name.as_str(), value)).unwrap())
            .collect::<Vec<BlockDisplay>>()
    }
}
