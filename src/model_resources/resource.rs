use std::fs;
use tracing::info;

use serde_json::Value;

use crate::{resource_error::create_resource_error, utils::get_optional_field};

use super::{
    multipart::Multipart,
    resource_error::{ResourceError, ResourceErrorKind},
    variant::Variants,
};

#[derive(Debug, Clone)]
pub enum ResourcePath {
    BlockModel(String),
    Texture(String),
    BlockState(String),
}

pub enum ModelVariants {
    MultipartVariant(Multipart),
    StandardVariant(Variants),
}

impl ModelVariants {
    pub(crate) fn load_from_json_in(path: &str) -> Result<ModelVariants, ResourceError> {
        info!("loading block state from disk: {}", path);
        let file = fs::read_to_string(path);
        let Ok(file) = file else {
            return Err(ResourceError {
                file: path.to_string(),
                kind: crate::resource_error::ResourceErrorKind::ErrorLoadingFile,
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

        Ok(match get_optional_field(&value, "variants") {
            Some(variants) => ModelVariants::StandardVariant(create_resource_error(path, || {
                Variants::from_json_value(variants)
            })?),
            None => match get_optional_field(&value, "multipart") {
                Some(multipart) => {
                    ModelVariants::MultipartVariant(create_resource_error(path, || {
                        Multipart::try_from_json(multipart)
                    })?)
                }
                None => {
                    return Err(ResourceError {
                        file: path.to_string(),
                        kind: ResourceErrorKind::MissingField(
                            "No variant or multipart field".to_string(),
                        ),
                    })
                }
            },
        })
    }
}
