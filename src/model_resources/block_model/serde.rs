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
    parent: Option<&'a str>,
    #[serde(default)]
    ambient_occlusion: bool,
    #[serde(default)]
    display: HashMap<DisplayPosition, PositionData>,
    #[serde(default)]
    #[serde(borrow)]
    textures: HashMap<&'a str, &'a str>,
    #[serde(default)]
    elements: Vec<RawElement<'a>>,
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
    from: [f32; 3],
    to: [f32; 3],
    #[serde(default)]
    rotation: Option<Rotation>,
    #[serde(default)]
    shade: bool,
    #[serde(default)]
    light_emission: i32,
    #[serde(default)]
    #[serde(borrow)]
    faces: HashMap<FaceName, RawFaceData<'a>>,
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
    uv: [f32; 4],
    #[serde(borrow)]
    texture: &'a str,
    #[serde(default)]
    cullface: Option<FaceName>,
    #[serde(default)]
    rotation: i32,
    #[serde(default = "default_tint")]
    tintindex: i32,
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
        let file =
            include_str!("../../../test_assets/assets/minecraft/models/block/bamboo4_age0.json");
        let a: Result<RawBlockModel<'static>, serde_json::Error> = serde_json::de::from_str(file);
        let b = a.map_err(|err| eprintln!("{err:?}")).unwrap();
    }

    #[test]
    fn test_mc_models() {
        let read_dir = std::fs::read_dir("./test_assets/assets/minecraft/models/block/").unwrap();

        for entry in read_dir.flatten() {
            let path = entry.path();

            if path.is_file() && path.extension().unwrap() == "json" {
                let file = std::fs::read_to_string(path).unwrap();

                let a: Result<RawBlockModel<'_>, serde_json::Error> =
                    serde_json::de::from_str(file.as_str());

                let b = a.map_err(|err| eprintln!("{err:?}")).unwrap();
            }
        }
    }
}
