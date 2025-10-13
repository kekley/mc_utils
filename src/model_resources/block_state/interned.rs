use std::slice;

use hashbrown::HashMap;
use lasso::{Rodeo, Spur};

use crate::block_state::serde::{Apply, BlockStateType, ModelProperties, RawVariantType, When};

pub enum InternedModelResult<'a> {
    SingleModel(&'a [InternedModelProperties]),
    Multipart(Vec<&'a InternedVariantType>),
}
#[derive(Debug)]
pub enum InternedBlockVariants {
    Variants(HashMap<Spur, InternedVariantType>),
    Multipart(Vec<InternedCase>),
}

impl InternedBlockVariants {
    pub fn get_model_properties_for_mapped_state(
        &self,
        mapped_state_spur: Spur,
        mapped_state_str: &str,
        interner: &Rodeo,
    ) -> Option<InternedModelResult<'_>> {
        match self {
            InternedBlockVariants::Variants(hash_map) => {
                let Some(variant_type) = hash_map.get(&mapped_state_spur) else {
                    eprintln!("mapped state {mapped_state_str} not found in variant map");
                    return None;
                };
                match variant_type {
                    InternedVariantType::SingleModel(interned_model_properties) => {
                        Some(InternedModelResult::SingleModel(slice::from_ref(
                            interned_model_properties,
                        )))
                    }
                    InternedVariantType::MultiModel(items) => {
                        Some(InternedModelResult::SingleModel(items))
                    }
                }
            }
            InternedBlockVariants::Multipart(interned_cases) => {
                Some(InternedModelResult::Multipart(
                    interned_cases
                        .iter()
                        .filter(|case| case.test_variant_string(mapped_state_str, interner))
                        .map(|case| case.get_models())
                        .collect::<Vec<_>>(),
                ))
            }
        }
    }
}

impl InternedBlockVariants {
    pub fn intern_blockstate(blockstate: BlockStateType<'_>, interner: &mut Rodeo) -> Self {
        match blockstate {
            BlockStateType::Variants(hash_map) => {
                let interned_map: HashMap<Spur, InternedVariantType> = hash_map
                    .iter()
                    .map(|(properties, variants)| {
                        let variants = match variants {
                            RawVariantType::SingleVariant(model_properties) => {
                                InternedVariantType::SingleModel(InternedModelProperties::intern(
                                    model_properties,
                                    interner,
                                ))
                            }
                            RawVariantType::MultiVariant(items) => {
                                let interned_items = items
                                    .iter()
                                    .map(|model_properties| {
                                        InternedModelProperties::intern(model_properties, interner)
                                    })
                                    .collect();
                                InternedVariantType::MultiModel(interned_items)
                            }
                        };
                        (interner.get_or_intern(properties), variants)
                    })
                    .collect();
                InternedBlockVariants::Variants(interned_map)
            }
            BlockStateType::Multipart(cases) => InternedBlockVariants::Multipart(
                cases
                    .iter()
                    .map(|case| {
                        let when = case.when().map(|when| match when.condition() {
                            When::Or(hash_maps) => InternedWhen::Or(
                                hash_maps
                                    .iter()
                                    .map(|hashmap| {
                                        hashmap
                                            .iter()
                                            .map(|(name, prop)| {
                                                (
                                                    interner.get_or_intern(name),
                                                    interner.get_or_intern(prop),
                                                )
                                            })
                                            .collect()
                                    })
                                    .collect(),
                            ),
                            When::And(hash_maps) => InternedWhen::And(
                                hash_maps
                                    .iter()
                                    .map(|hashmap| {
                                        hashmap
                                            .iter()
                                            .map(|(name, prop)| {
                                                (
                                                    interner.get_or_intern(name),
                                                    interner.get_or_intern(prop),
                                                )
                                            })
                                            .collect()
                                    })
                                    .collect(),
                            ),
                            When::Single(hash_map) => InternedWhen::SingleState(
                                hash_map
                                    .iter()
                                    .map(|(name, prop)| {
                                        (interner.get_or_intern(name), interner.get_or_intern(prop))
                                    })
                                    .collect(),
                            ),
                        });
                        let apply = match case.apply() {
                            Apply::Single(model_properties) => InternedApply {
                                inner: InternedVariantType::SingleModel(
                                    InternedModelProperties::intern(model_properties, interner),
                                ),
                            },
                            Apply::Many(items) => InternedApply {
                                inner: InternedVariantType::MultiModel(
                                    items
                                        .iter()
                                        .map(|model_properties| {
                                            InternedModelProperties::intern(
                                                model_properties,
                                                interner,
                                            )
                                        })
                                        .collect(),
                                ),
                            },
                        };

                        InternedCase { when, apply }
                    })
                    .collect(),
            ),
        }
    }
}

///A Variant type can have a single model or multiple models from which one is chosen at random
#[derive(Debug, Clone)]
pub enum InternedVariantType {
    SingleModel(InternedModelProperties),
    MultiModel(Vec<InternedModelProperties>),
}

#[derive(Debug, Clone)]
pub struct InternedModelProperties {
    pub(crate) model: Spur,
    pub(crate) x: i32,
    pub(crate) y: i32,
    pub(crate) uvlock: bool,
    pub(crate) weight: i32,
}

impl InternedModelProperties {
    pub fn intern(model_properties: &ModelProperties<'_>, interner: &mut Rodeo) -> Self {
        let model = interner.get_or_intern(model_properties.model());
        let x = model_properties.x_rotation();
        let y = model_properties.y_rotation();
        let uvlock = model_properties.uvlock();
        let weight = model_properties.weight();

        InternedModelProperties {
            model,
            x,
            y,
            uvlock,
            weight,
        }
    }
    pub fn get_x_rotation(&self) -> i32 {
        self.x
    }
    pub fn get_y_rotation(&self) -> i32 {
        self.y
    }
    pub fn get_uvlock(&self) -> bool {
        self.uvlock
    }
    pub fn get_weight(&self) -> i32 {
        self.weight
    }
    pub fn get_model_location_spur(&self) -> Spur {
        self.model
    }
}

fn default_weight() -> i32 {
    1
}

#[derive(Debug)]
pub struct InternedCase {
    pub(crate) when: InternedWhen,
    pub(crate) apply: InternedApply,
}
impl InternedCase {
    pub fn test_variant_string(&self, variant_str: &str, rodeo: &Rodeo) -> bool {
        self.when.test_variant_string(variant_str, rodeo)
    }
    pub fn get_models(&self) -> &InternedVariantType {
        &self.apply.inner
    }
}

#[derive(Debug)]
pub struct InternedApply {
    inner: InternedVariantType,
}

#[derive(Debug)]
pub enum InternedWhen {
    Or(Vec<HashMap<Spur, Spur>>),
    And(Vec<HashMap<Spur, Spur>>),
    SingleState(HashMap<Spur, Spur>),
    Empty,
}

impl InternedWhen {
    pub fn test_variant_string(&self, variant_string: &str, rodeo: &Rodeo) -> bool {
        match self {
            InternedWhen::Or(hash_maps) => Self::or_case(hash_maps, variant_string, rodeo),
            InternedWhen::And(hash_maps) => Self::and_case(hash_maps, variant_string, rodeo),
            InternedWhen::SingleState(hash_map) => {
                Self::single_case(hash_map.iter(), variant_string, rodeo)
            }
            InternedWhen::Empty => true,
        }
    }
    fn or_case(cases: &[HashMap<Spur, Spur>], variant_string: &str, rodeo: &Rodeo) -> bool {
        cases.iter().any(|case| {
            let iter = case.iter();
            Self::single_case(iter, variant_string, rodeo)
        })
    }

    fn and_case(cases: &[HashMap<Spur, Spur>], variant_string: &str, rodeo: &Rodeo) -> bool {
        cases.iter().all(|case| {
            let iter = case.iter();
            Self::single_case(iter, variant_string, rodeo)
        })
    }

    fn single_case<'a>(
        case: impl Iterator<Item = (&'a Spur, &'a Spur)>,
        variant_string: &str,
        rodeo: &Rodeo,
    ) -> bool {
        let variant_properties = variant_string.split(",");
        case.map(|(test_property_name_spur, test_property_value_spur)| {
            let test_property_name_str = rodeo.resolve(test_property_name_spur);
            let test_property_value_str = rodeo.resolve(test_property_value_spur);
            (test_property_name_str, test_property_value_str)
        })
        .all(|(test_property_name, test_property_value)| {
            variant_properties
                .clone()
                .map(|name_and_value| {
                    name_and_value
                        .split_once("=")
                        .unwrap_or((name_and_value, ""))
                })
                .any(|(variant_property_name, variant_property_value)| {
                    variant_property_name == test_property_name
                        && test_property_value
                            .split("|")
                            .any(|test_value| test_value == variant_property_value)
                })
        })
    }
}

#[cfg(test)]
mod tests {
    use lasso::Rodeo;

    use crate::block_state::{interned::InternedBlockVariants, serde::BlockStateType};

    #[test]
    fn test_interned_mc_blockstates() {
        let read_dir = std::fs::read_dir("./test_assets/assets/minecraft/blockstates/").unwrap();

        for entry in read_dir.flatten() {
            let path = entry.path();

            if path.is_file() && path.extension().unwrap() == "json" {
                let file = std::fs::read_to_string(path).unwrap();

                let a: Result<BlockStateType<'_>, serde_json::Error> =
                    serde_json::de::from_str(file.as_str());

                let b = a.map_err(|err| eprintln!("{err:?}")).unwrap();
                let mut interner = Rodeo::new();

                let c = InternedBlockVariants::intern_blockstate(b, &mut interner);
            }
        }
    }
}
