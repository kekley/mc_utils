use super::{block_states::BlockState, variant::Variant};

pub struct MultiPart {}

pub struct Case {
    when: Option<When>,
    apply: Apply,
}

pub struct Apply {
    variant: Variant,
}

pub enum When {
    OrCase(Vec<BlockState>),
    AndCase(Vec<BlockState>),
    SingleCase(Vec<BlockState>),
}
