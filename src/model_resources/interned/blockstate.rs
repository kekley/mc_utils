use hashbrown::HashMap;
use lasso::{Rodeo, Spur};

use crate::serde::blockstate::{BlockStateType, ModelProperties};

#[derive(Debug)]
pub enum InternedBlockState {
    Variants(HashMap<Spur, InternedVariantType>),
    Multipart(Vec<InternedCase>),
}

impl InternedBlockState {
    pub fn intern_blockstate(blockstate: BlockStateType<'_>, interner: &mut Rodeo) -> Self {
        match blockstate {
            BlockStateType::Variants(hash_map) => {
                let interned_map: HashMap<Spur, InternedVariantType> = hash_map
                    .iter()
                    .map(|(properties, variants)| {
                        let variants = match variants {
                            crate::serde::blockstate::VariantType::SingleVariant(
                                model_properties,
                            ) => InternedVariantType::SingleVariant(
                                InternedModelProperties::intern(model_properties, interner),
                            ),
                            crate::serde::blockstate::VariantType::MultiVariant(items) => {
                                let interned_items = items
                                    .iter()
                                    .map(|model_properties| {
                                        InternedModelProperties::intern(model_properties, interner)
                                    })
                                    .collect();
                                InternedVariantType::MultiVariant(interned_items)
                            }
                        };
                        (interner.get_or_intern(properties), variants)
                    })
                    .collect();
                InternedBlockState::Variants(interned_map)
            }
            BlockStateType::Multipart(cases) => InternedBlockState::Multipart(
                cases
                    .iter()
                    .map(|case| {
                        let when = case.when().map(|when| match when.condition() {
                            crate::serde::blockstate::When::Or(hash_maps) => InternedWhen::Or(
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
                            crate::serde::blockstate::When::And(hash_maps) => InternedWhen::And(
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
                            crate::serde::blockstate::When::Single(hash_map) => {
                                InternedWhen::SingleState(
                                    hash_map
                                        .iter()
                                        .map(|(name, prop)| {
                                            (
                                                interner.get_or_intern(name),
                                                interner.get_or_intern(prop),
                                            )
                                        })
                                        .collect(),
                                )
                            }
                        });
                        let apply = match case.apply() {
                            crate::serde::blockstate::Apply::Single(model_properties) => {
                                InternedApply::Single(InternedModelProperties::intern(
                                    model_properties,
                                    interner,
                                ))
                            }
                            crate::serde::blockstate::Apply::Many(items) => InternedApply::Many(
                                items
                                    .iter()
                                    .map(|model_properties| {
                                        InternedModelProperties::intern(model_properties, interner)
                                    })
                                    .collect(),
                            ),
                        };

                        InternedCase { when, apply }
                    })
                    .collect(),
            ),
        }
    }
}

#[derive(Debug)]
pub enum InternedVariantType {
    SingleVariant(InternedModelProperties),
    MultiVariant(Vec<InternedModelProperties>),
}

#[derive(Debug)]
pub struct InternedModelProperties {
    model: Spur,
    x: i32,
    y: i32,
    uvlock: bool,
    weight: i32,
}

impl InternedModelProperties {
    pub fn intern(model_properties: &ModelProperties<'_>, interner: &mut Rodeo) -> Self {
        let model = interner.get_or_intern(model_properties.model());
        let x = model_properties.x();
        let y = model_properties.y();
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
}

fn default_weight() -> i32 {
    1
}

#[derive(Debug)]
pub struct InternedCase {
    when: Option<InternedWhen>,
    apply: InternedApply,
}

#[derive(Debug)]
pub enum InternedApply {
    Single(InternedModelProperties),
    Many(Vec<InternedModelProperties>),
}

#[derive(Debug)]
pub enum InternedWhen {
    Or(Vec<HashMap<Spur, Spur>>),
    And(Vec<HashMap<Spur, Spur>>),
    SingleState(HashMap<Spur, Spur>),
}

#[cfg(test)]
mod tests {
    use lasso::Rodeo;

    use crate::{interned::blockstate::InternedBlockState, serde::blockstate::BlockStateType};

    #[test]
    fn test_interned_mc_blockstates() {
        let read_dir = std::fs::read_dir("./test_assets/assets/minecraft/blockstates/").unwrap();

        for entry in read_dir.flatten() {
            let path = entry.path();

            if path.is_file() && path.extension().unwrap() == "json" {
                println!("{path}", path = path.display());
                let file = std::fs::read_to_string(path).unwrap();

                let a: Result<BlockStateType<'_>, serde_json::Error> =
                    serde_json::de::from_str(file.as_str());

                let b = a.map_err(|err| eprintln!("{err:?}")).unwrap();
                let mut interner = Rodeo::new();

                let c = InternedBlockState::intern_blockstate(b, &mut interner);
            }
        }
    }
}
