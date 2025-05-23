use bumpalo::Bump;
use serde_json::Value;

use super::{resource_error::ResourceErrorKind, utils::parse_array};

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
    type Error = ResourceErrorKind;

    fn try_from(value: (&str, &Value)) -> Result<Self, Self::Error> {
        let position = value.0;
        let data = value.1;
        let rotation: Option<Result<[f32; 3], _>> =
            data.get("rotation").map(|f| parse_array::<f32, 3>(f));
        let translation: Option<Result<[f32; 3], ResourceErrorKind>> =
            data.get("translation").map(|f| parse_array::<f32, 3>(f));
        let scale: Option<Result<[f32; 3], ResourceErrorKind>> =
            data.get("scale").map(|f| parse_array::<f32, 3>(f));
        let res = BlockDisplay {
            position: position.into(),
            rotation: rotation.unwrap_or(Ok([0f32; 3]))?,
            translation: translation.unwrap_or(Ok([0f32; 3]))?,
            scale: scale.unwrap_or(Ok([1f32; 3]))?,
        };

        Ok(res)
    }
}

impl BlockDisplay {
    pub fn parse_display<'a>(
        value: &Value,
        bump: &'a mut Bump,
    ) -> Result<bumpalo::collections::Vec<'a, BlockDisplay>, ResourceErrorKind> {
        match value.as_object() {
            Some(object) => {
                let obj_iter = object
                    .iter()
                    .map(|(name, value)| BlockDisplay::try_from((name.as_str(), value)));
                Ok(bumpalo::collections::Vec::from_iter_in(obj_iter, bump))
            }
            None => Err(ResourceErrorKind::InvalidField(format!(
                "Block display field must be a json object. json value: {}",
                value.to_string()
            ))),
        }
    }
}
