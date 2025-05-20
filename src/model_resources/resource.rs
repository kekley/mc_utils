use std::fs;

use anyhow::{anyhow, Context};
use log::{error, info};
use serde_json::Value;

use crate::MCResourceLoader;

use super::{multipart::Multipart, variant::Variants};

#[derive(Debug, Clone)]
pub enum BlockStates {
    MultiPart(Multipart),
    Variants(Variants),
}

impl BlockStates {
    pub fn new(path: &str, loader: &MCResourceLoader) -> anyhow::Result<BlockStates> {
        info!("loading block state from disk: {}", path);
        let file = fs::read_to_string(path);
        let Ok(file) = file else {
            error!("could not find file {}", path);
            return Err(anyhow!("file not found"));
        };

        let value: Value = serde_json::from_str(&file).context("invalid json")?;
        if let Some(value) = value.get("variants") {
            Ok(BlockStates::Variants(Variants::from_json_value(
                value, &loader,
            )?))
        } else if let Some(value) = value.get("multipart") {
            Ok(BlockStates::MultiPart(Multipart::new(value, loader)?))
        } else {
            error!("Not a valid variant or multipart");
            return Err(anyhow!("failed parsing multipart"));
        }
    }
}
