use std::{hash::Hash, sync::RwLock};

use dashmap::DashMap;
use fxhash::FxBuildHasher;
pub type EntryID = u32;

#[derive(Debug)]
pub struct Palette<T: Eq + PartialEq + Hash + Clone> {
    map: DashMap<T, EntryID, FxBuildHasher>,
    entries: RwLock<Vec<T>>,
}

impl<T: Eq + Hash + Clone> Palette<T> {
    pub fn new() -> Self {
        let map = DashMap::with_hasher(FxBuildHasher::default());
        let entries = RwLock::new(vec![]);
        Self {
            map,
            entries: entries,
        }
    }

    fn contains(&self, key: &T) -> bool {
        self.map.contains_key(key)
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

impl<T: Eq + Hash + Clone> IntoIterator for Palette<T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.entries.into_inner().unwrap().into_iter()
    }
}
