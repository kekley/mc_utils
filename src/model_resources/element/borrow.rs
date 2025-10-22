use crate::face::{borrow::Face, common::rotation::Rotation};

#[derive(Debug, Clone)]
pub struct Element<'data> {
    pub from: [f32; 3],
    pub to: [f32; 3],
    pub rotation: Option<Rotation>,
    pub shade: bool,
    pub light_emission: i32,
    pub faces: [Option<Face<'data>>; 6],
}

impl Element<'_> {
    pub fn get_from(&self) -> [f32; 3] {
        self.from
    }

    pub fn get_to(&self) -> [f32; 3] {
        self.to
    }

    pub fn get_rotation(&self) -> Option<&Rotation> {
        self.rotation.as_ref()
    }

    pub fn get_light_emission(&self) -> i32 {
        self.light_emission
    }

    pub fn get_faces(&self) -> impl Iterator<Item = &Face<'_>> {
        self.faces.iter().filter_map(|option| {
            if let Some(face) = option {
                Some(face)
            } else {
                None
            }
        })
    }
}
