use hashbrown::HashMap;
use serde::Deserialize;

use crate::{
    block_model::common::{DisplayPosition, PositionData},
    face::common::{face_name::FaceName, rotation::Rotation},
};

#[derive(Deserialize, Debug)]
pub struct RawBlockModel<'a> {
    #[serde(default)]
    #[serde(borrow)]
    pub(crate) parent: Option<&'a str>,
    #[serde(default)]
    pub(crate) ambient_occlusion: bool,
    #[serde(default)]
    pub(crate) display: HashMap<DisplayPosition, PositionData>,
    #[serde(default)]
    #[serde(borrow)]
    pub(crate) textures: HashMap<&'a str, &'a str>,
    #[serde(default)]
    pub(crate) elements: Vec<RawElement<'a>>,
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
    pub fn elements(&self) -> &[RawElement<'a>] {
        &self.elements
    }

    #[inline]
    pub fn textures(&self) -> impl Iterator<Item = (&&'a str, &&'a str)> {
        self.textures.iter()
    }
}

#[derive(Deserialize, Debug)]
pub struct RawElement<'a> {
    pub(crate) from: [f32; 3],
    pub(crate) to: [f32; 3],
    #[serde(default)]
    pub(crate) rotation: Option<Rotation>,
    #[serde(default)]
    pub(crate) shade: bool,
    #[serde(default)]
    pub(crate) light_emission: i32,
    #[serde(default)]
    #[serde(borrow)]
    pub(crate) faces: HashMap<FaceName, RawFaceData<'a>>,
}

impl<'a> RawElement<'a> {
    #[inline]
    pub fn from(&self) -> [f32; 3] {
        self.from
    }

    #[inline]
    pub fn to(&self) -> [f32; 3] {
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

    pub fn faces(&self) -> impl Iterator<Item = (&FaceName, &RawFaceData<'a>)> {
        self.faces.iter()
    }
}

#[derive(Deserialize, Debug)]
pub struct RawFaceData<'a> {
    #[serde(default = "default_uv")]
    pub(crate) uv: [f32; 4],
    #[serde(borrow)]
    pub(crate) texture: &'a str,
    #[serde(default)]
    pub(crate) cullface: Option<FaceName>,
    #[serde(default)]
    pub(crate) rotation: i32,
    #[serde(default = "default_tint")]
    pub(crate) tintindex: i32,
}

impl<'a> RawFaceData<'a> {
    pub fn uv(&self) -> [f32; 4] {
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

fn default_uv() -> [f32; 4] {
    [0.0, 0.0, 16.0, 16.0]
}

fn default_tint() -> i32 {
    -1
}

#[cfg(test)]
mod tests {
    use crate::block_model::serde::RawBlockModel;

    #[test]
    fn test_block_model() {
        const TEST_MODEL: &str = r##"{
    "textures": {
        "particle": "#texture"
    },
    "elements": [
        {
            "name": "top bar",
            "from": [7, 12, 7],
            "to": [16, 15, 9],
            "faces": {
                "north": {"uv": [4, 4, 13, 7], "texture": "#texture"},
                "east": {"uv": [13, 4, 15, 7], "texture": "#texture", "cullface": "east"},
                "south": {"uv": [4, 4, 13, 7], "texture": "#texture"},
                "up": {"uv": [13, 7, 15, 16], "rotation": 270, "texture": "#texture"},
                "down": {"uv": [13, 7, 15, 16], "rotation": 90, "texture": "#texture"}
            }
        },
        {
            "name": "lower bar",
            "from": [7, 6, 7],
            "to": [16, 9, 9],
            "faces": {
                "north": {"uv": [4, 4, 13, 7], "texture": "#texture"},
                "east": {"uv": [13, 4, 15, 7], "texture": "#texture", "cullface": "east"},
                "south": {"uv": [4, 4, 13, 7], "texture": "#texture"},
                "up": {"uv": [13, 7, 15, 16], "rotation": 270, "texture": "#texture"},
                "down": {"uv": [13, 7, 15, 16], "rotation": 90, "texture": "#texture"}
            }
        }
    ],
    "groups": [
        {
            "name": "east",
            "origin": [0, 0, 0],
            "color": 0,
            "children": [0, 1]
        }
    ]
}"##;
        let a: Result<RawBlockModel<'static>, serde_json::Error> =
            serde_json::de::from_str(TEST_MODEL);
        let _b = a.map_err(|err| eprintln!("{err:?}")).unwrap();
    }

    #[test]
    fn test_mc_models() {
        const MODEL_DIR: &str = "../resource_pack/assets/minecraft/models/block/";
        let read_dir = std::fs::read_dir(MODEL_DIR).unwrap();

        for entry in read_dir.flatten() {
            let path = entry.path();

            if path.is_file() && path.extension().unwrap() == "json" {
                let file = std::fs::read_to_string(path).unwrap();

                let a: Result<RawBlockModel<'_>, serde_json::Error> =
                    serde_json::de::from_str(file.as_str());

                let _b = a.map_err(|err| eprintln!("{err:?}")).unwrap();
            }
        }
    }
}
