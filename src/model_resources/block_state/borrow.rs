use hashbrown::HashMap;

use crate::block_state::common::BlockRotation;

mod conversion {
    use lasso::{Rodeo, RodeoResolver};

    use crate::block_state::interned::{
        InternedApply, InternedBlockVariants, InternedCase, InternedModelProperties,
        InternedVariantType, InternedWhen,
    };

    use super::*;
    fn from_interned_variants<'data>(
        interned: &InternedBlockVariants,
        interner: &'data RodeoResolver,
    ) -> BlockVariants<'data> {
        match interned {
            InternedBlockVariants::Variants(hash_map) => from_variants(hash_map, interner),
            InternedBlockVariants::Multipart(interned_cases) => todo!(),
        }
    }

    fn from_multipart<'data>(
        cases: &[InternedCase],
        interner: &'data RodeoResolver,
    ) -> BlockVariants<'data> {
        BlockVariants::Multipart(
            cases
                .iter()
                .map(|case| from_case(case, interner))
                .collect::<Vec<_>>(),
        )
    }

    fn from_case<'data>(case: &InternedCase, interner: &'data RodeoResolver) -> Case<'data> {
        let InternedCase { when, apply } = case;

        Case {
            when: from_when(when, interner),
            apply: (),
        }
    }

    fn from_when<'data>(when: &InternedWhen, interner: &'data RodeoResolver) -> When<'data> {
        todo!()
    }
    fn from_apply<'data>(apply: &InternedApply, interner: &'data RodeoResolver) -> Apply<'data> {
        todo!()
    }

    fn from_variants<'data>(
        hash_map: &HashMap<lasso::Spur, crate::block_state::interned::InternedVariantType>,
        interner: &'data RodeoResolver,
    ) -> BlockVariants<'data> {
        let new_map = hash_map
            .iter()
            .map(|(k, v)| {
                let new_key = interner.resolve(k);
                let new_val = from_variant_types(v, interner);

                (new_key, new_val)
            })
            .collect::<HashMap<_, _>>();
        BlockVariants::Variants(new_map)
    }

    fn from_variant_types<'data>(
        types: &InternedVariantType,
        interner: &'data RodeoResolver,
    ) -> VariantType<'data> {
        match types {
            InternedVariantType::SingleModel(interned_model_properties) => {
                VariantType::SingleModel(from_model_properties(interned_model_properties, interner))
            }
            InternedVariantType::MultiModel(items) => VariantType::MultiModel(
                items
                    .iter()
                    .map(|properties| from_model_properties(properties, interner))
                    .collect::<Vec<_>>(),
            ),
        }
    }

    fn from_model_properties<'data>(
        model_properties: &InternedModelProperties,
        interner: &'data RodeoResolver,
    ) -> ModelProperties<'data> {
        let InternedModelProperties {
            model,
            x,
            y,
            uvlock,
            weight,
        } = model_properties;

        ModelProperties {
            model_resource_path: interner.resolve(&model),
            x_rotation: BlockRotation::try_from(x).unwrap(),
            y_rotation: BlockRotation::try_from(y).unwrap(),
            uvlock: *uvlock,
            weight: *weight,
        }
    }
}

///A structure for getting the "resource path" for a block model from a block's data
#[derive(Debug, Clone)]
pub enum BlockVariants<'data> {
    Variants(HashMap<&'data str, VariantType<'data>>),
    Multipart(Vec<Case<'data>>),
}

pub enum ModelResult<'variants, 'data> {
    SingleModel(&'variants [ModelProperties<'data>]),
    Multipart(Vec<&'variants VariantType<'data>>),
}

///A Variant type can have a single model or multiple models from which one is chosen at random
#[derive(Debug, Clone)]
pub enum VariantType<'data> {
    SingleModel(ModelProperties<'data>),
    MultiModel(Vec<ModelProperties<'data>>),
}

///A "Case" consists of a "When" clause and a model to "Apply" when that clause is met
#[derive(Debug, Clone)]
pub struct Case<'data> {
    when: When<'data>,
    apply: Apply<'data>,
}

///A list of blockstates that are to be matched for the "When" clause to be met
#[derive(Debug, Clone)]
pub enum When<'data> {
    Or(WhenStateList<'data>),
    And(WhenStateList<'data>),
    SingleState(&'data str, &'data str),
    Empty,
}

impl When<'_> {
    pub fn test_variant_string(&self, variant_str: &str) -> bool {
        match self {
            When::Or(when_state_list) => when_state_list.or_case(variant_str),
            When::And(when_state_list) => when_state_list.and_case(variant_str),
            When::SingleState(name, prop) => {
                WhenStateList::test_single_case(std::iter::once((*name, *prop)), variant_str)
            }
            When::Empty => true,
        }
    }
}

///The model to be applied when a "When" clause is met
#[derive(Debug, Clone)]
pub enum Apply<'data> {
    Single(ModelProperties<'data>),
    Many(Vec<ModelProperties<'data>>),
}

#[derive(Debug, Clone)]
pub struct ModelProperties<'data> {
    model_resource_path: &'data str,
    x_rotation: BlockRotation,
    y_rotation: BlockRotation,
    uvlock: bool,
    weight: i32,
}

impl<'data> ModelProperties<'data> {
    pub fn get_resource_path(&self) -> &str {
        self.model_resource_path
    }
    pub fn get_block_rotation_x(&self) -> BlockRotation {
        self.x_rotation
    }
    pub fn get_block_rotation_y(&self) -> BlockRotation {
        self.y_rotation
    }
    pub fn get_uvlock(&self) -> bool {
        self.uvlock
    }
    pub fn get_weight(&self) -> i32 {
        self.weight
    }
}

#[derive(Debug, Clone)]
pub struct WhenStateList<'data> {
    data: Vec<WhenState<'data>>,
}

///Iterates through the properties in a single "blockstate" of the ones present in a "when" clause
pub struct CaseIter<'data, 'list> {
    data: &'list [WhenState<'data>],
}

impl<'a, 'list> Iterator for CaseIter<'a, 'list> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        match self.data.first()? {
            WhenState::End => None,
            WhenState::State(name, value) => {
                self.data = &self.data[1..];
                Some((name, value))
            }
        }
    }
}

impl<'data> WhenStateList<'data> {
    pub fn iter_states<'list>(&'list self) -> WhenStatesIter<'data, 'list> {
        WhenStatesIter { data: &self.data }
    }
    fn or_case(&self, variant_str: &str) -> bool {
        self.iter_states()
            .any(|case| Self::test_single_case(case, variant_str))
    }

    fn and_case(&self, variant_str: &str) -> bool {
        self.iter_states()
            .all(|case| Self::test_single_case(case, variant_str))
    }

    fn test_single_case<'a>(
        mut case: impl Iterator<Item = (&'a str, &'a str)>,
        variant_str: &str,
    ) -> bool {
        case.all(|(test_property_name, test_property_value)| {
            variant_str
                .split(",")
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

pub struct WhenStatesIter<'a, 'list> {
    data: &'list [WhenState<'a>],
}

impl<'a, 'list> Iterator for WhenStatesIter<'list, 'a>
where
    'a: 'list,
{
    type Item = CaseIter<'a, 'list>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.data.is_empty() {
            return None;
        }
        let end_pos = self
            .data
            .iter()
            .position(|state| match state {
                WhenState::End => true,
                WhenState::State(_, _) => false,
            })
            .expect("Invalid When clause iterator");

        let state_slice = &self.data[..end_pos + 1];

        self.data = self.data.get(end_pos + 1..).unwrap_or(&[]);

        Some(CaseIter { data: state_slice })
    }
}

///Either a (property_name, property_value) tuple or a sentinel for the end of the current
///"blockstate"
#[derive(Debug, Clone)]
enum WhenState<'a> {
    End,
    State(&'a str, &'a str),
}
