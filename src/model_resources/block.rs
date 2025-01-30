use super::{block_states::BlockState, variant::BlockName};

#[derive(Debug, PartialEq, Eq, Hash, Clone)]
pub struct Block {
    pub block_name: BlockName,
    pub block_state: BlockState,
}
