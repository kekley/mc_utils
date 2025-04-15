use crate::loaded_world::BlockNameInternal;

use super::block_states::BlockStateInternal;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockInternal {
    pub block_name: BlockNameInternal,
    pub block_state: BlockStateInternal,
}
