use hashbrown::HashMap;

use crate::{
    block_model::{
        borrow::BlockModel,
        serde::{RawBlockModel, RawElement, RawFaceData},
    },
    block_state::common::UniqueStrings,
    element::borrow::Element,
    face::{
        borrow::{Face, FaceData},
        common::face_name::FaceName,
    },
};

pub(crate) fn intern_block_model<'a>(
    model: RawBlockModel<'_>,
    strings: &'a UniqueStrings,
) -> BlockModel<'a> {
    let RawBlockModel {
        parent,
        ambient_occlusion,
        display,
        textures,
        elements,
    } = model;

    let parent = parent.map(|str| strings.get_or_intern(str));
    let elements = elements
        .into_iter()
        .map(|raw_element| intern_element(raw_element, strings))
        .collect::<Vec<_>>();
    let textures = textures
        .into_iter()
        .map(|f| (strings.get_or_intern(f.0), strings.get_or_intern(f.1)))
        .collect::<HashMap<_, _>>();

    BlockModel {
        parent,
        ambient_occlusion,
        display,
        textures,
        elements,
    }
}

fn intern_element<'a>(raw_element: RawElement<'_>, strings: &'a UniqueStrings) -> Element<'a> {
    let RawElement {
        from,
        to,
        rotation,
        shade,
        light_emission,
        mut faces,
    } = raw_element;

    let faces: [Option<Face<'a>>; 6] = (0..6)
        .map(|i| {
            let name = FaceName::from_usize(i).expect("0..6 should always give a valid facename");

            let face_data = faces.remove(&name)?;

            let data = intern_face_data(face_data, strings);

            Some(Face { name, data })
        })
        .collect::<Vec<_>>()
        .try_into()
        .expect("0..6 should always result in an array of size 6");
    Element {
        from,
        to,
        rotation,
        shade,
        light_emission,
        faces,
    }
}

fn intern_face_data<'a>(face_data: RawFaceData<'_>, strings: &'a UniqueStrings) -> FaceData<'a> {
    let RawFaceData {
        uv,
        texture,
        cullface,
        rotation,
        tintindex,
    } = face_data;

    let texture = strings.get_or_intern(texture);

    FaceData {
        uv,
        texture,
        cullface,
        rotation,
        tintindex,
    }
}
