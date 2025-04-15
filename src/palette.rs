use std::{hash::Hash, sync::RwLock};

use dashmap::DashMap;
use fxhash::FxBuildHasher;
use lasso::Rodeo;

use crate::{block_states::InternalBlockState, loaded_world::BlockName};
pub type EntryID = u32;

#[derive(Debug, Clone)]
pub struct BlockPalette {
    interner: Rodeo,
    block_states: Vec<(BlockName, InternalBlockState)>,
}

impl BlockPalette {
    pub fn new() -> Self {
        let interner = Rodeo::new();
        Self { interner }
    }

    fn contains(&self, block: &str) -> bool {
        self.interner.contains(block)
    }

    pub fn insert(&self, key: T) -> EntryID {
        if let Some(val) = self.map.get(&key) {
            return *val;
        } else {
            let mut entries = self.entries.write().unwrap();
            let new_id = entries.len() as u32;
            self.map.insert(key.clone(), new_id);
            entries.push(key.clone());
            new_id
        }
    }
    pub fn get(&self, id: EntryID) -> Option<T> {
        self.entries.read().unwrap().get(id as usize).cloned()
    }
}

impl IntoIterator for BlockPalette {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_inner().unwrap().into_iter()
    }
}
