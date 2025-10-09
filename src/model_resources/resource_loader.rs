use crate::block_state::interned::InternedCase;
use crate::block_state::interned::InternedVariantType;
use crate::block_state::interned::VariantModelType;
use std::{
    collections::VecDeque,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    slice,
    time::Instant,
};

use compact_str::CompactString;
use hashbrown::HashMap;
use lasso::{Rodeo, Spur};
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    block_model::{interned::InternedBlockModel, serde::RawBlockModel},
    block_state::{interned::InternedBlockVariants, serde::BlockStateType},
    borrow::nbt_string::NBTStr,
    error::spider_eye_error::SpiderEyeError,
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
    type Interned = InternedBlockVariants;
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
        InternedBlockVariants::intern_blockstate(self, interner)
    }
}

impl<'a> Resource<'a> for RawBlockModel<'a> {
    type Interned = InternedBlockModel;
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
        InternedBlockModel::intern_block_model(self, interner)
    }
}

impl InternedResource for Box<[u8]> {
    type View<'a> = Box<[u8]>;
}

impl InternedResource for InternedBlockModel {
    type View<'a> = RawBlockModel<'a>;
}

impl InternedResource for InternedBlockVariants {
    type View<'a> = BlockStateType<'a>;
}

pub struct LoadedResources {
    pub interner: Rodeo,
    pub textures: HashMap<CompactString, Box<[u8]>>,
    pub models: HashMap<CompactString, InternedBlockModel>,
    pub variants: HashMap<CompactString, InternedBlockVariants>,
}

impl LoadedResources {
    pub fn get_models_for_block_properties(
        &self,
        mapped_state: &NBTStr,
    ) -> Option<VariantModelType<'_>> {
        let mapped_state_str = mapped_state.to_str();
        let (resource_location, variant_string) = mapped_state_str.split_once("#")?;

        let Some(variants) = self.variants.get(resource_location) else {
            eprintln!("{resource_location} did not have any loaded variants");
            return None;
        };

        let variant_type = match variants {
            InternedBlockVariants::Variants(hash_map) => {
                self.get_variants(hash_map, variant_string)
            }
            InternedBlockVariants::Multipart(interned_cases) => {
                self.get_multiparts(interned_cases, variant_string)
            }
        };

        variant_type
    }

    fn get_variants<'a>(
        &self,
        variant_map: &'a HashMap<Spur, InternedVariantType>,
        variant_string: &str,
    ) -> Option<VariantModelType<'a>> {
        let spur = self.interner.get(variant_string)?;

        variant_map
            .get(&spur)
            .map(|interned_variant| match interned_variant {
                InternedVariantType::SingleVariant(interned_model_properties) => {
                    VariantModelType::SingleModel(slice::from_ref(interned_model_properties))
                }

                InternedVariantType::MultiVariant(items) => {
                    VariantModelType::SingleModel(items.as_slice())
                }
            })
    }
    fn get_multiparts<'a>(
        &self,
        interned_cases: &'a [InternedCase],
        variant_string: &str,
    ) -> Option<VariantModelType<'a>> {
        let models = interned_cases
            .iter()
            .filter(|case| case.test_variant_string(variant_string, &self.interner))
            .map(|case| case.get_models())
            .collect::<Vec<_>>();
        if models.is_empty() {
            eprintln!("no models from multipart");
            return None;
        }

        Some(VariantModelType::Multipart(models))
    }

    pub fn try_get_spur(&self, str: &str) -> Option<Spur> {
        self.interner.get(str)
    }
    pub fn get_texture_data(&self, resource_location: &str) -> Option<&[u8]> {
        self.textures.get(resource_location).map(|b| b.as_ref())
    }

    pub fn get_model_data(&self, resource_location: &str) -> Option<&InternedBlockModel> {
        self.models.get(resource_location)
    }

    pub fn get_variant_data(&self, resource_location: &str) -> Option<&InternedBlockVariants> {
        self.variants.get(resource_location)
    }

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

        let start = Instant::now();
        let models = LoadedResources::parse_and_intern(&mut interner, model_files);
        let end = Instant::now();
        println!("interned models: {:?}", end.duration_since(start));

        let start = Instant::now();
        let textures = texture_files;
        let end = Instant::now();
        println!("interned textures {:?}", end.duration_since(start));

        let start = Instant::now();
        let variants = LoadedResources::parse_and_intern(&mut interner, blockstate_files);
        let end = Instant::now();
        println!("interned variants {:?}", end.duration_since(start));

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

        let start = Instant::now();

        let result = file_queue
            .par_iter()
            .filter_map(|path| {
                let block_name = path.file_stem()?.to_str()?;
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
                resource_path.push_str(block_name);
                println!("{resource_path}");

                let data = fs::read(path).ok()?;

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
