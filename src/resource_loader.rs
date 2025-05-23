use aovec::Aovec;
use bumpalo::Bump;
use bytes::Bytes;
use fxhash::FxBuildHasher;
use log::error;
use palette::ConcurrentPalette;

use crate::{
    block::InternedBlock,
    block_models::{BlockModel, BlockModelParent, IntermediateBlockModel},
    chunk::Chunk,
    loaded_world::{InternedBlockName, World},
    nbt_compound::NBTCompound,
    variant::ModelVariant,
};

use super::{block_models::ASSET_PATH, resource::BlockStates};
pub type BlockStateIndex = usize;
pub struct MCResourceLoader {
    arena: Bump,
}

impl MCResourceLoader {
    pub fn load_block_states_interned(&self, block_name: &str) -> Option<BlockStates> {
        let split = block_name
            .split_once(":")
            .unwrap_or(("minecraft", block_name));
        let namespace = split.0;
        let block_name_split = split.1;
        let mut path = String::new();
        path.push_str(&ASSET_PATH);
        path.push_str(namespace);
        path.push_str("/");
        path.push_str("blockstates/");

        path.push_str(block_name_split);
        path.push_str(".json");

        let new_state = BlockStates::load_from_json(&path, &self);
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
            BlockStates::Variants(variants) => variants.get_model_variants(&block.properties),
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
        Self { arena: Bump::new() }
    }
    pub fn get_texture_path(&self, resource_path: &str) -> String {
        let (namespace, remaining_str) = resource_path
            .split_once(":")
            .unwrap_or(("minecraft", resource_path));
        //dbg!(namespace, remaining_str);

        let (model_type, remaining_str) = remaining_str
            .split_once("/")
            .expect("invalid path for parent");
        //dbg!(model_type, remaining_str);
        String::from(
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
