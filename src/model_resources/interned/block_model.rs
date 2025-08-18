use crate::serde::block_model::{DisplayPosition, FaceName, PositionData, RawBlockModel, Rotation};
use hashbrown::HashMap;
use lasso::{Rodeo, Spur};

#[derive(Debug)]
pub struct BlockModel {
    parent: Option<Spur>,
    ambient_occlusion: bool,
    display: HashMap<DisplayPosition, PositionData>,
    textures: HashMap<Spur, Spur>,
    elements: Vec<InternedElement>,
}

impl BlockModel {
    pub fn intern_block_model(block_model: RawBlockModel<'_>, interner: &mut Rodeo) -> Self {
        let parent = block_model
            .get_parent()
            .map(|str| interner.get_or_intern(str));
        let ambient_occlusion = block_model.ambient_occlusion();
        let display = block_model
            .display()
            .map(|(position, data)| (position.clone(), data.clone()))
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

#[derive(Debug)]
pub struct InternedElement {
    from: [f64; 3],
    to: [f64; 3],
    rotation: Option<Rotation>,
    shade: bool,
    light_emission: i32,
    faces: HashMap<FaceName, InternedFaceData>,
}

#[derive(Debug)]
pub struct InternedFaceData {
    uv: [f64; 4],
    texture: Spur,
    cullface: Option<FaceName>,
    rotation: i32,
    tintindex: i32,
}

#[cfg(test)]
mod tests {
    use std::hint::black_box;

    use lasso::Rodeo;

    use crate::{interned::block_model::BlockModel, serde::block_model::RawBlockModel};

    #[test]
    fn conversion_test() {
        let read_dir = std::fs::read_dir("./test_assets/assets/minecraft/models/block/").unwrap();
        let mut interner = Rodeo::new();

        for entry in read_dir.flatten() {
            let path = entry.path();

            if path.is_file() && path.extension().unwrap() == "json" {
                println!("{path}", path = path.display());
                let file = std::fs::read_to_string(path).unwrap();

                let a: Result<RawBlockModel<'_>, serde_json::Error> =
                    serde_json::de::from_str(file.as_str());

                let b = a.map_err(|err| eprintln!("{err:?}")).unwrap();

                println!("{b:?}");

                let c = BlockModel::intern_block_model(&b, &mut interner);
                black_box(&c);
            }
        }
    }
}
