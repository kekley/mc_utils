use std::rc::Rc;

use lasso::Rodeo;

use crate::{block::BlockInternal, block_states::BlockStateInternal};
pub type PaletteIndex = u32;

#[derive(Debug, Clone)]
pub enum InternerType {
    Internal(Rodeo),
    External(Rc<Rodeo>),
}

#[derive(Debug, Clone)]
pub struct BlockPalette {
    interner: InternerType,
    block_states: Vec<BlockInternal>,
}

impl BlockPalette {
    pub fn new_with_interner(interner: Rc<Rodeo>) -> Self {}
    pub fn new() -> Self {
        let interner = Rodeo::new();
        Self {
            interner,
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

    pub fn insert(&mut self, block_name: &str, properties: &str) -> PaletteIndex {
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
            let block_state = BlockStateInternal::from_str(properties, &mut self.interner);
            let block = BlockInternal {
                block_name: block_name_spur,
                block_state,
            };
            let index = self.block_states.len();
            self.block_states.push(block);
            return index as PaletteIndex;
        }
    }

    pub fn get(&self, ind: PaletteIndex) -> Option<&BlockInternal> {
        self.block_states.get(ind as usize)
    }
}
