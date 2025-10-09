use serde::Deserialize;

#[derive(Debug, Deserialize, Clone)]
pub struct PositionData {
    #[serde(default)]
    rotation: [f32; 3],
    translation: [f32; 3],
    #[serde(default = "default_scale")]
    scale: [f32; 3],
}

fn default_scale() -> [f32; 3] {
    [1.0, 1.0, 1.0]
}

#[derive(Deserialize, Debug, Eq, PartialEq, Hash, Clone, Copy)]
pub enum DisplayPosition {
    #[serde(alias = "thirdperson_righthand")]
    ThirdPersonRightHand,
    #[serde(alias = "thirdperson_lefthand")]
    ThirdPersonLeftHand,
    #[serde(alias = "firstperson_righthand")]
    FirstPersonRightHand,
    #[serde(alias = "firstperson_lefthand")]
    FirstPersonLeftHand,
    #[serde(alias = "gui")]
    Gui,
    #[serde(alias = "head")]
    Head,
    #[serde(alias = "ground")]
    Ground,
    #[serde(alias = "fixed")]
    Fixed,
}
