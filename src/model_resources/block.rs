use crate::loaded_world::BlockName;

use super::block_states::InternalBlockState;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Block {
    pub block_name: BlockName,
    pub block_state: InternalBlockState,
}
