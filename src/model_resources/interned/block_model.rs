use crate::serde::block_model::{DisplayPosition, FaceName, PositionData, RawBlockModel, Rotation};
use hashbrown::HashMap;
use lasso::{Rodeo, Spur};

#[derive(Debug, Clone)]
pub struct InternedBlockModel {
    pub parent: Option<Spur>,
    pub ambient_occlusion: bool,
    pub display: HashMap<DisplayPosition, PositionData>,
    pub textures: HashMap<Spur, Spur>,
    pub elements: Vec<InternedElement>,
}

impl InternedBlockModel {
    pub fn get_parent_location(&self) -> Option<Spur> {
        self.parent
    }
    pub fn get_textures(&self) -> &HashMap<Spur, Spur> {
        &self.textures
    }
    pub fn get_elements(&self) -> &[InternedElement] {
        &self.elements
    }

    pub fn intern_block_model(block_model: RawBlockModel<'_>, interner: &mut Rodeo) -> Self {
        let parent = block_model
            .get_parent()
            .map(|str| interner.get_or_intern(str));
        let ambient_occlusion = block_model.ambient_occlusion();
        let display = block_model
            .display()
            .map(|(position, data)| (*position, data.clone()))
            .collect();
        let textures = block_model
            .textures()
            .map(|(tex_var, tex)| (interner.get_or_intern(tex_var), interner.get_or_intern(tex)))
            .collect();

        let elements = block_model
            .elements()
            .iter()
            .map(|element| {
                let from = element.from();
                let to = element.to();
                let rotation = element.rotation().cloned();

                let shade = element.shade();

                let light_emission = element.light_emission();

                let faces = element
                    .faces()
                    .map(|(face_name, face_data)| {
                        let face_data = InternedFaceData {
                            uv: face_data.uv(),
                            texture: interner.get_or_intern(face_data.texture()),
                            cullface: face_data.cullface(),
                            rotation: face_data.rotation(),
                            tintindex: face_data.tintindex(),
                        };
                        (*face_name, face_data)
                    })
                    .collect();
                InternedElement {
                    from,
                    to,
                    rotation,
                    shade,
                    light_emission,
                    faces,
                }
            })
            .collect();
        Self {
            parent,
            ambient_occlusion,
            display,
            textures,
            elements,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InternedElement {
    from: [f32; 3],
    to: [f32; 3],
    rotation: Option<Rotation>,
    shade: bool,
    light_emission: i32,
    faces: HashMap<FaceName, InternedFaceData>,
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
    pub fn faces(&self) -> &HashMap<FaceName, InternedFaceData> {
        &self.faces
    }
}

#[derive(Debug, Clone)]
pub struct InternedFaceData {
    uv: [f32; 4],
    texture: Spur,
    cullface: Option<FaceName>,
    rotation: i32,
    tintindex: i32,
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

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use lasso::Rodeo;

    use crate::{interned::block_model::InternedBlockModel, serde::block_model::RawBlockModel};

    #[test]
    fn conversion_test() {
        let read_dir = std::fs::read_dir("./test_assets/assets/minecraft/models/block/").unwrap();
        let mut interner = Rodeo::new();

        for entry in read_dir.flatten() {
            let path = entry.path();

            if path.is_file() && path.extension().unwrap() == "json" {
                let file = std::fs::read_to_string(path).unwrap();

                let a: Result<RawBlockModel<'_>, serde_json::Error> =
                    serde_json::de::from_str(file.as_str());

                let b = a.map_err(|err| eprintln!("{err:?}")).unwrap();

                let c = InternedBlockModel::intern_block_model(b, &mut interner);
                black_box(&c);
            }
        }
    }
}
