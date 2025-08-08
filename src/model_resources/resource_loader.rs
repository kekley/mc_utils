use std::path::{Path, PathBuf};

use crate::error::spider_eye_error::SpiderEyeError;

const ASSET_FOLDER_NAMES: [&str; 3] = ["blockstates", "models", "textures"];

pub enum ResourceType {
    BlockStates(PathBuf),
    Models(PathBuf),
    Textures(PathBuf),
    Unknown(PathBuf),
}

impl ResourceType {
    pub fn try_from_path(path: PathBuf) -> ResourceType {
        if !path.is_dir() {
            return ResourceType::Unknown(path);
        }

        let folder_name = path.file_name();

        if let Some(folder_name) = folder_name {
            match str {
                "blockstates" => Some(ResourceType::BlockStates),
                "models" => Some(ResourceType::Models),
                "textures" => Some(ResourceType::Textures),
                _ => None,
            }
        }
    }
}

pub fn load_resource_folder(path: &str) -> Result<LoadedResources, SpiderEyeError> {
    let resource_folder = std::fs::read_dir(path)?;

    let namespaces: Vec<Namespace> = resource_folder
        .into_iter()
        .filter_map(|dir| {
            let entry = dir.ok()?;

            let path = entry.path();

            if !path.is_dir() {
                return None;
            }

            let inner_folders = std::fs::read_dir(&path).ok()?;

            let asset_folders: [Option<ResourceType>; 3] = [const { None }; 3];

            for folder in inner_folders {
                if let Ok(folder) = folder {
                    let path = folder.path();
                    ASSET_FOLDER_NAMES
                        .iter()
                        .find(|folder_name| path.ends_with(fo));
                }
            }

            todo!();
        })
        .collect();

    Ok(todo!())
}

struct Namespace {
    path: PathBuf,
    blockstates: Option<PathBuf>,
    models: Option<PathBuf>,
    textures: Option<PathBuf>,
}

pub struct LoadedResources {}
