use std::fs;

use bumpalo::Bump;
use log::{error, info};
use serde_json::Value;

use super::{multipart::Multipart, resource_error::ResourceError, variant::Variants};

pub enum ResourcePath<'a> {
    BlockModel(bumpalo::collections::String<'a>),
    Texture(bumpalo::collections::String<'a>),
    BlockState(bumpalo::collections::String<'a>),
}

#[derive(Debug, Clone)]
pub enum BlockStates<'a> {
    MultiPart(Multipart),
    Variants(Variants),
}

impl<'a> BlockStates<'a> {
    pub fn load_from_json(path: &str, bump: &mut Bump) -> Result<BlockStates<'a>, ResourceError> {
        info!("loading block state from disk: {}", path);
        let file = fs::read_to_string(path);
        let Ok(file) = file else {
            return Err(ResourceError {
                file: path.to_string(),
                kind: crate::resource_error::ResourceErrorKind::FileNotFound,
            });
        };

        let value: Value = match serde_json::from_str(&file) {
            Ok(value) => value,
            Err(err) => {
                return Err(ResourceError {
                    file: path.to_string(),
                    kind: crate::resource_error::ResourceErrorKind::InvalidJSON(err),
                })
            }
        };

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
