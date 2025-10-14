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
