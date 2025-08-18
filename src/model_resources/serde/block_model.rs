use hashbrown::HashMap;
use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct RawBlockModel<'a> {
    #[serde(default)]
    #[serde(borrow)]
    parent: Option<&'a str>,
    #[serde(default)]
    ambient_occlusion: bool,
    #[serde(default)]
    display: HashMap<DisplayPosition, PositionData>,
    #[serde(default)]
    #[serde(borrow)]
    textures: HashMap<&'a str, &'a str>,
    #[serde(default)]
    elements: Vec<Element<'a>>,
}

impl<'a> RawBlockModel<'a> {
    #[inline]
    pub fn get_parent(&self) -> Option<&'a str> {
        self.parent
    }

    #[inline]
    pub fn ambient_occlusion(&self) -> bool {
        self.ambient_occlusion
    }
    pub fn display(&self) -> impl Iterator<Item = (&DisplayPosition, &PositionData)> {
        self.display.iter()
    }

    #[inline]
    pub fn elements(&self) -> &[Element<'a>] {
        &self.elements
    }

    #[inline]
    pub fn textures(&self) -> impl Iterator<Item = (&&'a str, &&'a str)> {
        self.textures.iter()
    }
}

#[derive(Debug, Deserialize, Clone)]
pub struct PositionData {
    #[serde(default)]
    rotation: [f64; 3],
    translation: [f64; 3],
    #[serde(default = "default_scale")]
    scale: [f64; 3],
}

fn default_scale() -> [f64; 3] {
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

#[derive(Deserialize, Debug)]
pub struct Element<'a> {
    from: [f64; 3],
    to: [f64; 3],
    #[serde(default)]
    rotation: Option<Rotation>,
    #[serde(default)]
    shade: bool,
    #[serde(default)]
    light_emission: i32,
    #[serde(default)]
    #[serde(borrow)]
    faces: HashMap<FaceName, FaceData<'a>>,
}

impl<'a> Element<'a> {
    #[inline]
    pub fn from(&self) -> [f64; 3] {
        self.from
    }

    #[inline]
    pub fn to(&self) -> [f64; 3] {
        self.to
    }

    pub fn rotation(&self) -> Option<&Rotation> {
        self.rotation.as_ref()
    }
    pub fn shade(&self) -> bool {
        self.shade
    }

    pub fn light_emission(&self) -> i32 {
        self.light_emission
    }

    pub fn faces(&self) -> impl Iterator<Item = (&FaceName, &FaceData<'a>)> {
        self.faces.iter()
    }
}

#[derive(Deserialize, Debug, Hash, PartialEq, Eq, Clone, Copy)]
pub enum FaceName {
    #[serde(alias = "down")]
    Down = 2,

    #[serde(alias = "up")]
    Up = 3,

    #[serde(alias = "north")]
    North = 4,

    #[serde(alias = "south")]
    South = 5,

    #[serde(alias = "west")]
    West = 0,

    #[serde(alias = "east")]
    East = 1,
}

#[derive(Deserialize, Debug)]
pub struct FaceData<'a> {
    #[serde(default = "default_uv")]
    uv: [f64; 4],
    #[serde(borrow)]
    texture: &'a str,
    #[serde(default)]
    cullface: Option<FaceName>,
    #[serde(default)]
    rotation: i32,
    #[serde(default = "default_tint")]
    tintindex: i32,
}

impl<'a> FaceData<'a> {
    pub fn uv(&self) -> [f64; 4] {
        self.uv
    }
    pub fn texture(&self) -> &'a str {
        self.texture
    }
    pub fn cullface(&self) -> Option<FaceName> {
        self.cullface
    }
    pub fn rotation(&self) -> i32 {
        self.rotation
    }
    pub fn tintindex(&self) -> i32 {
        self.tintindex
    }
}

fn default_uv() -> [f64; 4] {
    [0.0, 0.0, 16.0, 16.0]
}

fn default_tint() -> i32 {
    -1
}

#[derive(Deserialize, Debug, Clone)]
pub struct Rotation {
    origin: [f64; 3],
    axis: Axis,
    angle: f64,
    #[serde(default)]
    rescale: bool,
}

#[derive(Deserialize, Debug, Clone, Copy)]
pub enum Axis {
    #[serde(alias = "x")]
    X,

    #[serde(alias = "y")]
    Y,

    #[serde(alias = "z")]
    Z,
}
#[cfg(test)]
mod tests {
    use crate::serde::block_model::RawBlockModel;

    #[test]
    fn test_block_model() {
        let file =
            include_str!("../../../test_assets/assets/minecraft/models/block/bamboo4_age0.json");
        let a: Result<RawBlockModel<'static>, serde_json::Error> = serde_json::de::from_str(file);
        let b = a.map_err(|err| eprintln!("{err:?}")).unwrap();

        println!("{b:?}");
    }

    #[test]
    fn test_mc_models() {
        let read_dir = std::fs::read_dir("./test_assets/assets/minecraft/models/block/").unwrap();

        for entry in read_dir.flatten() {
            let path = entry.path();

            if path.is_file() && path.extension().unwrap() == "json" {
                println!("{path}", path = path.display());
                let file = std::fs::read_to_string(path).unwrap();

                let a: Result<RawBlockModel<'_>, serde_json::Error> =
                    serde_json::de::from_str(file.as_str());

                let b = a.map_err(|err| eprintln!("{err:?}")).unwrap();
            }
        }
    }
}
