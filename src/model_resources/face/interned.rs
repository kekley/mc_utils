use lasso::Spur;

use crate::face::common::face_name::FaceName;

#[derive(Debug, Clone)]
pub struct InternedFace {
    pub face_name: FaceName,
    pub data: InternedFaceData,
}

#[derive(Debug, Clone)]
pub struct InternedFaceData {
    pub uv: [f32; 4],
    pub texture: Spur,
    pub cullface: Option<FaceName>,
    pub rotation: i32,
    pub tintindex: i32,
}

impl InternedFaceData {
    pub fn uv(&self) -> &[f32; 4] {
        &self.uv
    }
    pub fn texture(&self) -> Spur {
        self.texture
    }
    pub fn rotation(&self) -> i32 {
        self.rotation
    }
    pub fn tint_index(&self) -> i32 {
        self.tintindex
    }
}
