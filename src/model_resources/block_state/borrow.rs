use hashbrown::HashMap;

use crate::block_state::common::BlockRotation;

///A structure for getting the "resource path" for a block model from a block's data
#[derive(Debug, Clone)]
pub enum BlockVariants<'data> {
    Variants(HashMap<&'data str, VariantType<'data>>),
    Multipart(Vec<Case<'data>>),
}

impl BlockVariants<'_> {
    pub fn get_models_for_block_properties<'a>(
        &'a self,
        variant_str: &str,
    ) -> Option<ModelResult<'a>> {
        Some(match self {
            BlockVariants::Variants(hash_map) => match hash_map.get(variant_str)? {
                VariantType::SingleModel(model_properties) => {
                    ModelResult::SingleModel(std::slice::from_ref(model_properties))
                }
                VariantType::MultiModel(items) => ModelResult::SingleModel(items),
            },
            BlockVariants::Multipart(cases) => ModelResult::Multipart(
                cases
                    .iter()
                    .filter(|case| case.when.test_variant_string(variant_str))
                    .map(|case| case.apply.to_slice())
                    .collect::<Vec<_>>(),
            ),
        })
    }
}

pub enum ModelResult<'data> {
    SingleModel(&'data [BlockModelInfo<'data>]),
    Multipart(Vec<&'data [BlockModelInfo<'data>]>),
}

///A Variant type can have a single model or multiple models from which one is chosen at random
#[derive(Debug, Clone)]
pub enum VariantType<'data> {
    SingleModel(BlockModelInfo<'data>),
    MultiModel(Vec<BlockModelInfo<'data>>),
}

///A "Case" consists of a "When" clause and a model to "Apply" when that clause is met
#[derive(Debug, Clone)]
pub struct Case<'data> {
    pub(crate) when: When<'data>,
    pub(crate) apply: Apply<'data>,
}

impl Case<'_> {
    pub fn test_variant_string(&self, variant_str: &str) -> bool {
        self.when.test_variant_string(variant_str)
    }

    pub(crate) fn get_models(&self) -> &[BlockModelInfo<'_>] {
        self.apply.to_slice()
    }
}

///A list of blockstates that are to be matched for the "When" clause to be met
#[derive(Debug, Clone)]
pub enum When<'data> {
    Or(WhenStateList<'data>),
    And(WhenStateList<'data>),
    SingleState(WhenStateList<'data>),
    Empty,
}

impl When<'_> {
    pub fn test_variant_string(&self, variant_str: &str) -> bool {
        match self {
            When::Or(when_state_list) => when_state_list.or_case(variant_str),
            When::And(when_state_list) => when_state_list.and_case(variant_str),
            When::SingleState(case) => {
                WhenStateList::test_single_case(case.iter_states().next().unwrap(), variant_str)
            }
            When::Empty => true,
        }
    }
}

///The model to be applied when a "When" clause is met
#[derive(Debug, Clone)]
pub enum Apply<'data> {
    Single(BlockModelInfo<'data>),
    Many(Vec<BlockModelInfo<'data>>),
}

impl Apply<'_> {
    pub fn to_slice(&self) -> &[BlockModelInfo<'_>] {
        match self {
            Apply::Single(model_properties) => std::slice::from_ref(model_properties),
            Apply::Many(items) => items,
        }
    }
}

#[derive(Debug, Clone)]
pub struct BlockModelInfo<'data> {
    pub(crate) model_resource_path: &'data str,
    pub(crate) x_rotation: BlockRotation,
    pub(crate) y_rotation: BlockRotation,
    pub(crate) uvlock: bool,
    pub(crate) weight: i32,
}

impl<'data> BlockModelInfo<'data> {
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
    pub(crate) data: Vec<WhenElement<'data>>,
}

///Iterates through the properties in a single "blockstate" of the ones present in a "when" clause
pub struct CaseIter<'data, 'list> {
    data: &'list [WhenElement<'data>],
}

impl<'a, 'list> Iterator for CaseIter<'a, 'list> {
    type Item = (&'a str, &'a str);

    fn next(&mut self) -> Option<Self::Item> {
        match self.data.first()? {
            WhenElement::End => None,
            WhenElement::Property(name, value) => {
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
    data: &'list [WhenElement<'a>],
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
                WhenElement::End => true,
                WhenElement::Property(_, _) => false,
            })
            .expect("Invalid When clause iterator");

        let state_slice = &self.data[..end_pos + 1];

        self.data = self.data.get(end_pos + 1..).unwrap_or(&[]);

        Some(CaseIter { data: state_slice })
    }
}

///Either a (`property_name`,`property_value`) tuple or a sentinel for the end of the current
///"blockstate"
#[derive(Debug, Clone)]
pub(crate) enum WhenElement<'a> {
    End,
    Property(&'a str, &'a str),
}
