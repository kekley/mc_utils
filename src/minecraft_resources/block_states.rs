use std::hash::Hash;

use hashbrown::HashMap;
use smol_str::SmolStr;

pub type BlockResource = SmolStr;
pub type StateName = SmolStr;
pub type State = SmolStr;

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct BlockState {
    pub block: BlockResource,
    pub properties: Option<HashMap<StateName, State>>,
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
