use hashbrown::HashMap;

use crate::block_state::{
    borrow::{
        Apply, BlockModelInfo, BlockVariants, Case, VariantType, When, WhenElement, WhenStateList,
    },
    common::{BlockRotation, UniqueStrings},
    serde::{
        OrAnd, RawApply, RawBlockVariants, RawCase, RawModelProperties, RawVariantType, WhenStruct,
    },
};

pub(crate) fn intern_blockstate_type<'a>(
    blockstate_type: RawBlockVariants<'_>,
    strings: &'a UniqueStrings,
) -> BlockVariants<'a> {
    match blockstate_type {
        RawBlockVariants::Variants(hash_map) => BlockVariants::Variants(
            hash_map
                .into_iter()
                .map(|(state_string, variant)| {
                    let interned = strings.get_or_intern(state_string);

                    (interned, intern_variants(variant, strings))
                })
                .collect(),
        ),
        RawBlockVariants::Multipart(cases) => BlockVariants::Multipart(
            cases
                .into_iter()
                .map(|case| intern_case(case, strings))
                .collect(),
        ),
    }
}

fn intern_case(case: RawCase<'_>, strings: &UniqueStrings) -> Case<'static> {
    let RawCase { when, apply } = case;
    let when = if let Some(when) = when {
        intern_when(when, strings)
    } else {
        When::Empty
    };

    Case {
        when,
        apply: intern_apply(apply, strings),
    }
}

fn intern_apply(apply: RawApply<'_>, strings: &UniqueStrings) -> Apply<'static> {
    match apply {
        RawApply::Single(raw_model_properties) => {
            Apply::Single(intern_properties(raw_model_properties, strings))
        }
        RawApply::Many(items) => Apply::Many(
            items
                .into_iter()
                .map(|properties| intern_properties(properties, strings))
                .collect(),
        ),
    }
}

fn intern_when(when: WhenStruct<'_>, strings: &UniqueStrings) -> When<'static> {
    if let Some(single_state) = when.single_state {
        When::SingleState(WhenStateList {
            data: intern_state_map(single_state, strings),
        })
    } else {
        match when.or_and.unwrap() {
            OrAnd::Or(hash_maps) => {
                let vec = hash_maps
                    .into_iter()
                    .flat_map(|state| {
                        let mut state_vec = intern_state_map(state, strings);
                        state_vec.push(WhenElement::End);
                        state_vec
                    })
                    .collect();

                When::Or(WhenStateList { data: vec })
            }
            OrAnd::And(hash_maps) => {
                let vec = hash_maps
                    .into_iter()
                    .flat_map(|state| {
                        let mut state_vec = intern_state_map(state, strings);
                        state_vec.push(WhenElement::End);
                        state_vec
                    })
                    .collect();
                When::And(WhenStateList { data: vec })
            }
        }
    }
}

fn intern_state_map<'a>(
    map: HashMap<&str, &str>,
    strings: &'a UniqueStrings,
) -> Vec<WhenElement<'a>> {
    map.into_iter()
        .map(|(a, b)| WhenElement::Property(strings.get_or_intern(a), strings.get_or_intern(b)))
        .collect()
}

fn intern_variants(variants: RawVariantType<'_>, strings: &UniqueStrings) -> VariantType<'static> {
    match variants {
        RawVariantType::SingleVariant(raw_model_properties) => {
            VariantType::SingleModel(intern_properties(raw_model_properties, strings))
        }
        RawVariantType::MultiVariant(items) => VariantType::MultiModel(
            items
                .into_iter()
                .map(|properties| intern_properties(properties, strings))
                .collect(),
        ),
    }
}

fn intern_properties(
    properties: RawModelProperties<'_>,
    strings: &UniqueStrings,
) -> BlockModelInfo<'static> {
    let RawModelProperties {
        model,
        x,
        y,
        uvlock,
        weight,
    } = properties;

    let interned = strings.get_or_intern(model);

    BlockModelInfo {
        model_resource_path: interned,
        x_rotation: BlockRotation::try_from(x).unwrap_or_default(),
        y_rotation: BlockRotation::try_from(y).unwrap_or_default(),
        uvlock,
        weight,
    }
}
