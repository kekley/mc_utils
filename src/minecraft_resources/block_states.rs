use serde_json::Value;

use super::block_models::BlockModel;

pub struct Variant {}

pub struct MultiPart {}
pub enum BlockStates {
    Variant(Vec<Variant>),
    MultiPart(),
}

pub struct BlockVariant<'a> {
    name: &'a str,
    model: BlockModel,
}

