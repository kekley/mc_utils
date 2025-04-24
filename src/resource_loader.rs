use std::sync::Arc;

use bytes::Bytes;
use lasso::{Spur, ThreadedRodeo};
use smol_str::{SmolStr, SmolStrBuilder};

use crate::{
    block::InternedBlock, chunk::Chunk, loaded_world::World, nbt_compound::NBTCompound,
    variant::ModelVariant,
};

use super::{block_models::ASSET_PATH, resource::BlockStates};

#[derive(Debug, Clone, Default)]
pub struct MCLoader {
    pub rodeo: Arc<ThreadedRodeo>,
}

impl MCLoader {
    pub fn load_block_states(&self, block_name: &str) -> BlockStates {
        let stripped_name = block_name.strip_prefix("minecraft:").unwrap();
        let mut path = SmolStrBuilder::new();
        path.push_str(&ASSET_PATH);

        path.push_str("minecraft/");
        path.push_str("blockstates/");

        path.push_str(stripped_name);
        path.push_str(".json");
        let path = path.finish();
        BlockStates::new(&path, &self.rodeo)
    }
    pub fn load_models(&self, block: &InternedBlock) -> Vec<ModelVariant> {
        let block_states = self.load_block_states(self.resolve_spur(&block.block_name));
        let variants = match block_states {
            BlockStates::MultiPart(multipart) => multipart.get(&block.properties),
            BlockStates::Variants(variants) => variants.get_model(&block.properties),
        };
        variants
    }
    pub fn open_world(&self, region_folder: &str) -> World {
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
