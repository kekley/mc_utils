use crate::face::common::face_name::FaceName;

#[derive(Debug, Clone)]
pub struct Face<'data> {
    pub(crate) name: FaceName,
    pub(crate) data: FaceData<'data>,
}

impl<'a> Face<'a> {
    pub fn get_name(&self) -> FaceName {
        self.name
    }
    pub fn get_uv(&self) -> &[f32; 4] {
        &self.data.uv
    }
    pub fn get_texture(&self) -> &str {
        self.data.texture
    }
    pub fn get_rotation(&self) -> &i32 {
        &self.data.rotation
    }
    pub fn get_tint_index(&self) -> &i32 {
        &self.data.tintindex
    }
}

#[derive(Debug, Clone)]
pub struct FaceData<'data> {
    pub uv: [f32; 4],
    pub texture: &'data str,
    pub cullface: Option<FaceName>,
    pub rotation: i32,
    pub tintindex: i32,
}
