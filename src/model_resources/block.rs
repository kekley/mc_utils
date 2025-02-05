use super::{block_states::InternalBlockState, variant::BlockName};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Block {
    pub block_name: BlockName,
    pub block_state: InternalBlockState,
}
