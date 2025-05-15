use std::sync::Arc;

use aovec::Aovec;
use bytes::Bytes;
use dashmap::DashMap;
use fxhash::FxBuildHasher;
use lasso::{Spur, ThreadedRodeo};
use smol_str::{SmolStr, SmolStrBuilder};

use crate::{
    block::InternedBlock,
    block_models::{IntermediateBlockModel, InternedBlockModel},
    chunk::Chunk,
    loaded_world::{InternedBlockName, World},
    nbt_compound::NBTCompound,
    variant::ModelVariant,
};

use super::{block_models::ASSET_PATH, resource::BlockStates};
pub type BlockStateIndex = usize;
pub struct MCResourceLoader {
    pub rodeo: Arc<ThreadedRodeo>,
    cached_block_models: Aovec<IntermediateBlockModel>,
    block_model_map: DashMap<Spur, Option<usize>, FxBuildHasher>,
    cached_block_states: Aovec<BlockStates>,
    block_state_map: DashMap<InternedBlockName, Option<BlockStateIndex>, FxBuildHasher>,
}

impl MCResourceLoader {
    fn load_model_file_cached(&self, resource_path: Spur) -> Option<&IntermediateBlockModel> {
        if let Some(index) = self.block_model_map.get(&resource_path) {
            return Some(&self.cached_block_models[index.clone()?]);
        }
        let str = self.rodeo.resolve(&resource_path);
        let path = IntermediateBlockModel::parent_to_path(str);
        let tmp = IntermediateBlockModel::from_json(&path, &self.rodeo);
        if let Some(model) = tmp {
            let index = self.cached_block_models.push(model);
            self.block_model_map.insert(resource_path, Some(index));
            self.cached_block_models.get(index)
        } else {
            self.block_model_map.insert(resource_path, None);
            return None;
        }
    }
    pub fn load_block_model(&self, resource_path: Spur) -> Option<InternedBlockModel> {
        let intermediate = self.load_model_file_cached(resource_path)?;
        self.collapse_parents(intermediate)
    }
    pub fn collapse_parents(&self, model: &IntermediateBlockModel) -> Option<InternedBlockModel> {
        if model.parent.is_some() {
            let parent = model.parent.as_ref().unwrap();
            let mut parent: IntermediateBlockModel = self.load_model_file_cached(*parent)?.clone();
            if model.elements.is_some() {
                parent.elements = model.elements.clone();
            }
            if model.textures.is_some() {
                if parent.textures.is_some() {
                    parent
                        .textures
                        .as_mut()
                        .unwrap()
                        .combine(model.textures.clone().unwrap());
                } else {
                    parent.textures = model.textures.clone();
                }
            }
            if model.displays.is_some() {
                parent.displays = model.displays.clone();
            }

            return self.collapse_parents(&parent);
        } else {
            return InternedBlockModel::try_from_intermediate(model);
        }
    }
    pub fn load_block_states_str(&self, block_name: &str) -> Option<&BlockStates> {
        let interned = self.rodeo.get_or_intern(block_name);
        self.load_block_states_interned(interned)
    }
    pub fn load_block_states_interned(
        &self,
        block_name: InternedBlockName,
    ) -> Option<&BlockStates> {
        if let Some(index) = self.block_state_map.get(&block_name) {
            return Some(&self.cached_block_states[index.clone()?]);
        }
        let block_name_str = self.rodeo.resolve(&block_name);
        let split = block_name_str
            .split_once(":")
            .unwrap_or(("minecraft", block_name_str));
        let namespace = split.0;
        let block_name_split = split.1;
        let mut path = SmolStrBuilder::new();
        path.push_str(&ASSET_PATH);
        path.push_str(namespace);
        path.push_str("/");
        path.push_str("blockstates/");

        path.push_str(block_name_split);
        path.push_str(".json");
        let path = path.finish();

        let new_state = BlockStates::new(&path, &self);
        if let Some(state) = new_state {
            let index = self.cached_block_states.len();
            self.cached_block_states.push(state);
            self.block_state_map.insert(block_name, Some(index));
            return Some(&self.cached_block_states[index]);
        } else {
            self.block_state_map.insert(block_name, None);
            return None;
        }
    }
    pub fn load_variants_for(
        &self,
        block: &InternedBlock,
        block_states: &BlockStates,
    ) -> Vec<ModelVariant> {
        let variants = match block_states {
            BlockStates::MultiPart(multipart) => {
                multipart.load_models(&block.properties, &self.rodeo)
            }
            BlockStates::Variants(variants) => variants.get_model(&block.properties),
        };
        variants
    }
    pub fn open_world(&self, region_folder: &str) -> anyhow::Result<World> {
        World::new(region_folder, &self.rodeo)
    }

    pub fn nbt_from_bytes(&self, bytes: &mut Bytes) -> anyhow::Result<NBTCompound> {
        NBTCompound::internal_nbt(bytes, &self.rodeo)
    }

    pub(crate) fn chunk_from_nbt(&self, chunk_nbt: NBTCompound) -> Option<Chunk> {
        Some(Chunk::from_nbt_internal(chunk_nbt, &self.rodeo))
    }

    pub fn new() -> Self {
        Self {
            rodeo: Arc::new(ThreadedRodeo::new()),
            cached_block_states: Aovec::new(16),
            block_state_map: Default::default(),
            cached_block_models: Aovec::new(16),
            block_model_map: Default::default(),
        }
    }
    pub fn get_texture_path(&self, resource_path: &str) -> SmolStr {
        let (namespace, remaining_str) = resource_path
            .split_once(":")
            .unwrap_or(("minecraft", resource_path));
        //dbg!(namespace, remaining_str);

        let (model_type, remaining_str) = remaining_str
            .split_once("/")
            .expect("invalid path for parent");
        //dbg!(model_type, remaining_str);
        SmolStr::from(
            ASSET_PATH.to_string()
                + namespace
                + "/"
                + "textures/"
                + model_type
                + "/"
                + remaining_str
                + ".png",
        )
    }
    pub fn resolve_spur(&self, spur: &Spur) -> &str {
        self.rodeo.resolve(spur)
    }
}
