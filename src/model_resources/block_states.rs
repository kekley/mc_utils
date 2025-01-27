use std::{hash::Hash, sync::Arc};

use hashbrown::HashMap;
use lasso::{Spur, ThreadedRodeo};
use smol_str::SmolStr;

pub type BlockResource = Spur;
pub type StateName = Spur;
pub type State = Spur;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockState {
    pub(crate) rodeo: Arc<ThreadedRodeo>,
    pub(crate) block: BlockResource,
    pub(crate) properties: Option<HashMap<StateName, State>>,
}

impl Hash for BlockState {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.block.hash(state);

        if let Some(properties) = &self.properties {
            for (key, value) in properties {
                key.hash(state);
                value.hash(state);
            }
        }
    }
}

impl BlockState {
    pub fn block_name(&self) -> &str {
        self.rodeo.resolve(&self.block)
    }
}
