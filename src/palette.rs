use std::sync::Arc;

use lasso::{Interner, Reader, Resolver, Rodeo, Spur, ThreadedRodeo};

use crate::{block::InternedBlock, block_states::InternedBlockState};
pub type PaletteIndex = u32;

impl Resolver for InternerType {
    fn resolve<'a>(&'a self, key: &Spur) -> &'a str {
        match self {
            InternerType::Internal(rodeo) => rodeo.resolve(key),
            InternerType::External(threaded_rodeo) => threaded_rodeo.resolve(key),
        }
    }

    fn try_resolve<'a>(&'a self, key: &Spur) -> Option<&'a str> {
        match self {
            InternerType::Internal(rodeo) => rodeo.try_resolve(key),
            InternerType::External(threaded_rodeo) => threaded_rodeo.try_resolve(key),
        }
    }

    unsafe fn resolve_unchecked<'a>(&'a self, key: &Spur) -> &'a str {
        match self {
            InternerType::Internal(rodeo) => rodeo.resolve_unchecked(key),
            InternerType::External(threaded_rodeo) => threaded_rodeo.resolve_unchecked(key),
        }
    }

    fn contains_key(&self, key: &Spur) -> bool {
        match self {
            InternerType::Internal(rodeo) => rodeo.contains_key(key),
            InternerType::External(threaded_rodeo) => threaded_rodeo.contains_key(key),
        }
    }

    fn len(&self) -> usize {
        match self {
            InternerType::Internal(rodeo) => rodeo.len(),
            InternerType::External(threaded_rodeo) => threaded_rodeo.len(),
        }
    }
}

impl Reader for InternerType {
    fn get(&self, val: &str) -> Option<Spur> {
        match self {
            InternerType::Internal(rodeo) => rodeo.get(val),
            InternerType::External(threaded_rodeo) => threaded_rodeo.get(val),
        }
    }

    fn contains(&self, val: &str) -> bool {
        match self {
            InternerType::Internal(rodeo) => rodeo.contains(val),
            InternerType::External(threaded_rodeo) => threaded_rodeo.contains(val),
        }
    }
}

impl Interner for InternerType {
    fn get_or_intern(&mut self, val: &str) -> Spur {
        match self {
            InternerType::Internal(rodeo) => rodeo.get_or_intern(val),
            InternerType::External(threaded_rodeo) => threaded_rodeo.get_or_intern(val),
        }
    }

    fn try_get_or_intern(&mut self, val: &str) -> lasso::LassoResult<Spur> {
        match self {
            InternerType::Internal(rodeo) => rodeo.try_get_or_intern(val),
            InternerType::External(threaded_rodeo) => threaded_rodeo.try_get_or_intern(val),
        }
    }

    fn get_or_intern_static(&mut self, val: &'static str) -> Spur {
        match self {
            InternerType::Internal(rodeo) => rodeo.get_or_intern_static(val),
            InternerType::External(threaded_rodeo) => threaded_rodeo.get_or_intern_static(val),
        }
    }

    fn try_get_or_intern_static(&mut self, val: &'static str) -> lasso::LassoResult<Spur> {
        match self {
            InternerType::Internal(rodeo) => rodeo.try_get_or_intern_static(val),
            InternerType::External(threaded_rodeo) => threaded_rodeo.try_get_or_intern_static(val),
        }
    }
}

#[derive(Debug, Clone)]
pub enum InternerType {
    Internal(Rodeo),
    External(Arc<ThreadedRodeo>),
}

#[derive(Debug, Clone)]
pub struct BlockPalette {
    pub interner: Arc<ThreadedRodeo>,
    block_states: Vec<InternedBlock>,
}

impl BlockPalette {
    pub fn new_inner(interner: &Arc<ThreadedRodeo>) -> Self {
        Self {
            interner: interner.clone(),
            block_states: vec![],
        }
    }

    pub fn contains(&self, block_name: &str) -> bool {
        self.block_states.iter().any(|block_internal| {
            let name_str = self.interner.resolve(&block_internal.block_name);
            if name_str == block_name {
                return true;
            } else {
                return false;
            }
        })
    }
    pub fn insert_block(&mut self, block_: InternedBlock) -> PaletteIndex {
        let a = self
            .block_states
            .iter()
            .enumerate()
            .find(|(_, block)| block == block);
        if let Some(value) = a {
            return value.0 as PaletteIndex;
        } else {
            let ind = self.block_states.len();
            self.block_states.push(block_);
            return ind as PaletteIndex;
        }
    }
    pub fn insert_str(&mut self, block_name: &str, properties: &str) -> PaletteIndex {
        if self.interner.contains(block_name) {
            let (index, _) = self
                .block_states
                .iter()
                .enumerate()
                .find(|(_, block_internal)| {
                    let name_str = self.interner.resolve(&block_internal.block_name);
                    if name_str == block_name {
                        return true;
                    } else {
                        return false;
                    }
                })
                .unwrap();
            return index as PaletteIndex;
        } else {
            let block_name_spur = self.interner.get_or_intern(block_name);
            let block_state = InternedBlockState::from_str(properties, &mut self.interner);
            let block = InternedBlock {
                block_name: block_name_spur,
                properties: block_state,
            };
            let index = self.block_states.len();
            self.block_states.push(block);
            return index as PaletteIndex;
        }
    }

    pub fn get(&self, ind: PaletteIndex) -> Option<&InternedBlock> {
        self.block_states.get(ind as usize)
    }
}
