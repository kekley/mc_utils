use std::{
    collections::VecDeque,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
};

use compact_str::CompactString;
use hashbrown::HashMap;

use crate::error::spider_eye_error::SpiderEyeError;

use super::{
    block_models::{BlockModel, IntermediateBlockModel},
    resource::ModelVariants,
};

const ASSET_FOLDER_NAMES: [&str; 3] = ["blockstates", "models", "textures"];

#[derive(Debug, Clone, Copy)]
pub enum ResourceType {
    BlockStates,
    Models,
    Textures,
}

impl ResourceType {
    pub fn to_extension(&self) -> &'static str {
        match self {
            ResourceType::BlockStates => "json",
            ResourceType::Models => "json",
            ResourceType::Textures => "png",
        }
    }
    pub fn to_asset_type(&self) -> &'static str {
        match self {
            ResourceType::BlockStates => "blockstates",
            ResourceType::Models => "models",
            ResourceType::Textures => "textures",
        }
    }
    pub fn from_path(path: PathBuf) -> Option<ResourceType> {
        if !path.is_dir() {
            return None;
        }

        let folder_name = path.file_name();

        if let Some(os_str) = folder_name {
            if let Some(folder_name) = os_str.to_str() {
                return match folder_name {
                    "blockstates" => Some(ResourceType::BlockStates),
                    "models" => Some(ResourceType::Models),
                    "textures" => Some(ResourceType::Textures),
                    _ => None,
                };
            }
        }
        None
    }
}

pub fn load_resource_folder(path: &Path) -> Result<LoadedResources, SpiderEyeError> {
    let resource_folder = std::fs::read_dir(path)?;

    let mut all_textures: HashMap<CompactString, Box<[u8]>> = Default::default();

    let mut all_models: HashMap<CompactString, IntermediateBlockModel> = Default::default();

    let mut all_blockstates: HashMap<CompactString, ModelVariants> = Default::default();

    for folder in resource_folder.flatten() {
        let namespace_path = folder.path();

        if let Some(textures) =
            LoadedResources::traverse_folder::<Box<[u8]>>(&namespace_path, &ResourceType::Textures)
        {
            all_textures.extend(textures);
        }

        if let Some(models) = LoadedResources::traverse_folder::<IntermediateBlockModel>(
            &namespace_path,
            &ResourceType::Models,
        ) {
            all_models.extend(models);
        }

        if let Some(block_states) = LoadedResources::traverse_folder::<ModelVariants>(
            &namespace_path,
            &ResourceType::BlockStates,
        ) {
            all_blockstates.extend(block_states);
        }
    }

    dbg!(&all_textures.len());

    dbg!(&all_models.len());
    dbg!(&all_blockstates.len());

    Ok(LoadedResources {
        textures: all_textures,
        models: all_models,
        variants: all_blockstates,
    })
}

pub struct Namespace {
    path: PathBuf,
}

impl Namespace {
    fn from_arr(path: PathBuf, mut folders: [Option<ResourceType>; 3]) -> Option<Self> {
        if folders.iter().any(|f| f.is_none()) {
            return None;
        }

        Some(Self { path })
    }
}

pub trait Resource {
    type Output: Debug;

    fn load(path: &Path) -> Option<Self::Output>;
}

impl Resource for Box<[u8]> {
    type Output = Box<[u8]>;
    fn load(path: &Path) -> Option<Self::Output> {
        Some(std::fs::read(path).ok()?.into_boxed_slice())
    }
}

impl Resource for ModelVariants {
    type Output = ModelVariants;

    fn load(path: &Path) -> Option<Self::Output> {
        ModelVariants::load_from_json(path).ok()
    }
}

impl Resource for IntermediateBlockModel {
    type Output = IntermediateBlockModel;
    fn load(path: &Path) -> Option<Self::Output> {
        IntermediateBlockModel::from_json(path).ok()
    }
}

pub struct LoadedResources {
    textures: HashMap<CompactString, Box<[u8]>>,
    models: HashMap<CompactString, IntermediateBlockModel>,
    variants: HashMap<CompactString, ModelVariants>,
}

impl LoadedResources {
    pub fn traverse_folder<T: Resource>(
        folder: &Path,
        resource_type: &ResourceType,
    ) -> Option<HashMap<CompactString, T::Output>> {
        let namespace = folder.file_name()?;

        let mut resource_type_path = folder.to_path_buf();

        resource_type_path.push(resource_type.to_asset_type());

        println!("collecting all {type:?} for {namespace}",type =resource_type, namespace = namespace.display());

        let mut result = HashMap::new();

        let mut folder_traversal_queue = VecDeque::new();

        if let Ok(entries) = fs::read_dir(folder) {
            folder_traversal_queue.extend(
                entries
                    .flatten()
                    .map(|dir_entry| dir_entry.path())
                    .filter(|path| path.is_dir()),
            );
        }

        while !folder_traversal_queue.is_empty() {
            let current_folder = folder_traversal_queue
                .pop_front()
                .expect("Queue should not be empty");

            if let Ok(dir_entries) = std::fs::read_dir(current_folder) {
                for entry in dir_entries.flatten() {
                    let path = entry.path();

                    if path.is_dir() {
                        folder_traversal_queue.push_back(path);
                    } else if path.is_file() {
                        if let Some(extension) = path.extension() {
                            if extension != resource_type.to_extension() {
                                continue;
                            }
                        }
                        let mut resource_path = CompactString::new(namespace.to_str()?);
                        resource_path.push(':');

                        resource_path.push('/');
                        if let Ok(remainder) = path.strip_prefix(folder) {
                            if let Some(path_str) = remainder.to_str() {
                                resource_path.push_str(path_str);
                                if let Some(resource) = T::load(&path) {
                                    result.insert(resource_path, resource);
                                }
                            }
                        }
                    }
                }
            }
        }
        Some(result)
    }
}
