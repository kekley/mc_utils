use std::{
    collections::VecDeque,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};

use compact_str::CompactString;
use hashbrown::HashMap;
use lasso::Rodeo;

use crate::{
    error::spider_eye_error::SpiderEyeError,
    interned::{block_model::BlockModel, blockstate::InternedBlockState},
    serde::{block_model::RawBlockModel, blockstate::BlockStateType},
};

const _ASSET_FOLDER_NAMES: [&str; 3] = ["blockstates", "models", "textures"];

#[derive(Debug, Clone, Copy)]
pub enum ResourceType {
    BlockStates,
    Models,
    Textures,
}

impl ResourceType {
    pub fn to_extension(&self) -> &'static str {
        match self {
            ResourceType::BlockStates | ResourceType::Models => "json",
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

pub trait InternedResource {
    type View<'a>: Resource<'a, Interned = Self>;
}

pub trait Resource<'a>: Send + Debug + Sized {
    type Interned: InternedResource;
    fn load(data: &'a mut [u8]) -> Option<Self>;
    fn folder_name() -> &'static str;
    fn extension() -> &'static str;
    fn intern(self, interner: &mut Rodeo) -> Self::Interned;
}

impl Resource<'_> for Box<[u8]> {
    type Interned = Box<[u8]>;
    fn load(data: &mut [u8]) -> Option<Self> {
        Some(data.to_vec().into_boxed_slice())
    }

    fn folder_name() -> &'static str {
        "textures"
    }

    fn extension() -> &'static str {
        "png"
    }

    fn intern(self, _interner: &mut Rodeo) -> Self::Interned {
        self
    }
}

impl<'a> Resource<'a> for BlockStateType<'a> {
    type Interned = InternedBlockState;
    fn load(data: &'a mut [u8]) -> Option<BlockStateType<'a>> {
        simd_json::serde::from_slice(data).ok()?
    }

    fn folder_name() -> &'static str {
        "blockstates"
    }

    fn extension() -> &'static str {
        "json"
    }

    fn intern(self, interner: &mut Rodeo) -> Self::Interned {
        InternedBlockState::intern_blockstate(self, interner)
    }
}

impl<'a> Resource<'a> for RawBlockModel<'a> {
    type Interned = BlockModel;
    fn load(data: &'a mut [u8]) -> Option<RawBlockModel<'a>> {
        simd_json::serde::from_slice(data).ok()?
    }

    fn folder_name() -> &'static str {
        "models"
    }

    fn extension() -> &'static str {
        "json"
    }

    fn intern(self, interner: &mut Rodeo) -> Self::Interned {
        BlockModel::intern_block_model(self, interner)
    }
}

impl InternedResource for Box<[u8]> {
    type View<'a> = Box<[u8]>;
}

impl InternedResource for BlockModel {
    type View<'a> = RawBlockModel<'a>;
}

impl InternedResource for InternedBlockState {
    type View<'a> = BlockStateType<'a>;
}

pub struct LoadedResources {
    pub interner: Rodeo,
    pub textures: HashMap<CompactString, Box<[u8]>>,
    pub models: HashMap<CompactString, BlockModel>,
    pub variants: HashMap<CompactString, InternedBlockState>,
}

impl LoadedResources {
    pub fn load_resource_folder(path: &Path) -> Result<LoadedResources, SpiderEyeError> {
        let resource_folder = std::fs::read_dir(path)?;
        let mut texture_files: HashMap<CompactString, Box<[u8]>> = Default::default();

        let mut model_files: HashMap<CompactString, Box<[u8]>> = Default::default();

        let mut blockstate_files: HashMap<CompactString, Box<[u8]>> = Default::default();

        resource_folder.flatten().for_each(|dir_entry| {
            let namespace_path = dir_entry.path();

            println!("{namespace_path:?}");

            println!("textures");
            if let Some(textures) = LoadedResources::traverse_and_load::<Box<[u8]>>(&namespace_path)
            {
                texture_files.extend(textures);
            }

            println!("models");
            if let Some(models) =
                LoadedResources::traverse_and_load::<RawBlockModel<'static>>(&namespace_path)
            {
                model_files.extend(models);
            }

            println!("blockstates");
            if let Some(block_states) =
                LoadedResources::traverse_and_load::<BlockStateType<'static>>(&namespace_path)
            {
                blockstate_files.extend(block_states);
            }
        });

        let mut interner = Rodeo::new();

        let models = LoadedResources::parse_and_intern(&mut interner, model_files);

        let textures = texture_files;
        let variants = LoadedResources::parse_and_intern(&mut interner, blockstate_files);

        Ok(Self {
            interner,
            textures,
            models,
            variants,
        })
    }

    fn traverse_and_load<'a, 'folder_path, T: Resource<'a>>(
        folder: &'folder_path Path,
    ) -> Option<HashMap<CompactString, Box<[u8]>>> {
        let namespace = folder.file_name().unwrap();
        let mut resource_type_path = folder.to_path_buf();

        resource_type_path.push(T::folder_name());

        let mut folder_traversal_queue = VecDeque::new();

        let mut file_queue = Vec::new();

        println!("traversing folders");

        let start = Instant::now();

        if let Ok(entries) = fs::read_dir(folder) {
            folder_traversal_queue.extend(
                entries
                    .flatten()
                    .map(|dir_entry| dir_entry.path())
                    .filter(|path| path.is_dir()),
            );
        }

        while let Some(current_folder) = folder_traversal_queue.pop_front() {
            if let Ok(dir_entries) = std::fs::read_dir(current_folder) {
                for entry in dir_entries.flatten() {
                    if let Ok(file_type) = entry.file_type() {
                        if file_type.is_dir() {
                            let dir_path = entry.path();
                            folder_traversal_queue.push_back(dir_path);
                        } else if file_type.is_file() {
                            let file_path = entry.path();
                            if let Some(extension) = file_path.extension() {
                                if extension != T::extension() {
                                    continue;
                                }
                            }
                            file_queue.push(file_path);
                        }
                    }
                }
            }
        }

        let end = Instant::now();

        println!("time to traverse folders: {:?}", end.duration_since(start));

        println!("Loading start");

        let start = Instant::now();

        let result = file_queue
            .into_iter()
            .filter_map(|path| {
                let name = path.file_stem()?.to_str()?;
                let mut resource_path = CompactString::new(namespace.to_str()?);
                resource_path.push(':');

                let remainder = path
                    .strip_prefix(resource_type_path.as_path())
                    .ok()?
                    .parent()?;
                let path_str = remainder.to_str()?;
                resource_path.push_str(path_str);
                if !path_str.is_empty() {
                    resource_path.push('/');
                }
                resource_path.push_str(name);

                let data = fs::read(&path).ok()?;

                Some((resource_path, data.into_boxed_slice()))
            })
            .collect::<HashMap<_, _>>();

        let end = Instant::now();

        println!("time spent loading: {:?}", end.duration_since(start));

        Some(result)
    }

    fn parse_and_intern<T>(
        interner: &mut Rodeo,
        files: HashMap<CompactString, Box<[u8]>>,
    ) -> HashMap<CompactString, T>
    where
        T: InternedResource,
    {
        let resources: HashMap<CompactString, T> = files
            .into_iter()
            .filter_map(|(resource_path, mut data)| {
                let resource = T::View::load(data.as_mut())?;
                let interned = T::View::intern(resource, interner);

                Some((resource_path, interned))
            })
            .collect();
        resources
    }
}
