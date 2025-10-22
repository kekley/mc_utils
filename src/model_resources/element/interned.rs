use crate::face::{common::rotation::Rotation, interned::InternedFace};

#[derive(Debug, Clone)]
pub struct InternedElement {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub rotation: Option<Rotation>,
    pub shade: bool,
    pub light_emission: i32,
    pub faces: [Option<InternedFace>; 6],
}

impl InternedElement {
    pub fn from(&self) -> &[f32; 3] {
        &self.from
    }
    pub fn to(&self) -> &[f32; 3] {
        &self.to
    }
    pub fn rotation(&self) -> Option<&Rotation> {
        self.rotation.as_ref()
    }
    pub fn faces(&self) -> impl Iterator<Item = Option<&InternedFace>> {
        self.faces.iter().map(|f| f.as_ref())
    }
}
