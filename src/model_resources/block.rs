use std::sync::Arc;

use lasso::ThreadedRodeo;

use crate::{loaded_world::InternedBlockName, MCResourceLoader};

use super::block_states::{BlockState, InternedBlockState};

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct InternedBlock {
    pub block_name: InternedBlockName,
    pub properties: InternedBlockState,
}

impl InternedBlock {
    pub fn resolve<'a>(&'a self, loader: &'a MCResourceLoader) -> ResolvedBlock<'a> {
        let interner = &loader.rodeo;
        let name = interner.resolve(&self.block_name);

        let properties = self.properties.resolve(loader);

        ResolvedBlock {
            block_name: name,
            properties: properties,
        }
    }
}

#[derive(Debug)]
pub struct ResolvedBlock<'a> {
    pub block_name: &'a str,
    pub properties: BlockState<'a>,
}
