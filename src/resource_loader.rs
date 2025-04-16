use std::{fs, sync::Arc};

use bytes::Bytes;
use hashbrown::HashMap;
use lasso::{Spur, ThreadedRodeo};
use serde_json::Value;
use smol_str::{SmolStr, SmolStrBuilder};

use crate::{
    chunk::{self, Chunk},
    loaded_world::World,
    nbt_compound::NBTCompound,
    SpiderEyeError,
};

use super::{
    block_models::{BlockModel, ASSET_PATH},
    resource::BlockStates,
};

#[derive(Debug, Clone)]
pub struct ResourceLoader {
    pub rodeo: Arc<ThreadedRodeo<Spur>>,
}

impl ResourceLoader {
    pub fn load_block(&self, block_name: &str) -> BlockStates {
        let mut path = SmolStrBuilder::new();
        path.push_str(&ASSET_PATH);

        path.push_str("minecraft/");
        path.push_str("blockstates/");

        path.push_str(block_name);
        path.push_str(".json");
        let path = path.finish();
        BlockStates::new(&path, &self.rodeo)
    }
    pub fn load_model(&self, path: &str) -> BlockModel {
        BlockModel::load(path, &self.rodeo)
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
        //        dbg!(namespace, remaining_str);

        let (model_type, remaining_str) = remaining_str
            .split_once("/")
            .expect("invalid path for parent");
        //        dbg!(model_type, remaining_str);
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
}
