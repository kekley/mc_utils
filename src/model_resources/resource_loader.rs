use crate::block_model::intern::intern_block_model;
use crate::block_state::borrow::{BlockVariants, BlockstateType, Case, VariantType};
use crate::block_state::intern::intern_blockstate_type;
use crate::{block_model::borrow::BlockModel, block_state::common::UniqueStrings};
use hashbrown::HashMap;
use std::{
    collections::VecDeque,
    fmt::Debug,
    fs,
    path::{Path, PathBuf},
    time::Instant,
};
use thiserror::Error;
use tracing::{event, instrument, Level};

use compact_str::CompactString;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    block_model::serde::RawBlockModel, block_state::serde::RawBlockVariants,
    error::spider_eye_error::MCUtilsError,
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

trait InternedResource<'b> {
    type View<'a>: Resource<'a, Interned = Self>;
}

impl InternedResource<'_> for BlockVariants<'static> {
    type View<'a> = RawBlockVariants<'a>;
}

impl InternedResource<'_> for BlockModel<'static> {
    type View<'a> = RawBlockModel<'a>;
}

impl InternedResource<'_> for Box<[u8]> {
    type View<'a> = Box<[u8]>;
}

#[derive(Debug, Error)]
pub enum ResourceLoadError {
    #[error("Error parsing json: {0}")]
    SerdeError(#[from] simd_json::Error),
}

trait Resource<'a>: Send + Debug + Sized {
    type Interned;
    type LoadError;
    fn load(data: &'a mut [u8]) -> Result<Self, Self::LoadError>;
    fn folder_name() -> &'static str;
    fn extension() -> &'static str;
    fn intern(self, strings: &mut UniqueStrings) -> Self::Interned;
}

impl Resource<'_> for Box<[u8]> {
    type Interned = Box<[u8]>;
    type LoadError = ();
    fn load(data: &mut [u8]) -> Result<Self, ()> {
        Ok(data.to_vec().into_boxed_slice())
    }

    fn folder_name() -> &'static str {
        "textures"
    }

    fn extension() -> &'static str {
        "png"
    }

    fn intern(self, _strings: &mut UniqueStrings) -> Self::Interned {
        self
    }
}

impl<'a> Resource<'a> for RawBlockVariants<'a> {
    type Interned = BlockVariants<'static>;
    type LoadError = ResourceLoadError;
    fn load(data: &'a mut [u8]) -> Result<RawBlockVariants<'a>, ResourceLoadError> {
        Ok(simd_json::serde::from_slice::<RawBlockVariants<'_>>(data)?)
    }

    fn folder_name() -> &'static str {
        "blockstates"
    }

    fn extension() -> &'static str {
        "json"
    }

    fn intern(self, strings: &mut UniqueStrings) -> Self::Interned {
        intern_blockstate_type(self, strings)
    }
}

impl<'a> Resource<'a> for RawBlockModel<'a> {
    type Interned = BlockModel<'static>;
    type LoadError = ResourceLoadError;
    fn load(data: &'a mut [u8]) -> Result<RawBlockModel<'a>, ResourceLoadError> {
        Ok(simd_json::serde::from_slice(data)?)
    }

    fn folder_name() -> &'static str {
        "models"
    }

    fn extension() -> &'static str {
        "json"
    }

    fn intern(self, strings: &mut UniqueStrings) -> Self::Interned {
        intern_block_model(self, strings)
    }
}

#[derive(Debug)]
pub struct ResourceLoader {
    //The backing store for the strings from deserialized JSON
    _strings: UniqueStrings,
    textures: HashMap<CompactString, Box<[u8]>>,
    models: HashMap<CompactString, BlockModel<'static>>,
    variants: HashMap<CompactString, BlockVariants<'static>>,
}

impl ResourceLoader {
    pub fn load_resource_folder(path: &Path) -> Result<ResourceLoader, MCUtilsError> {
        let resource_folder = std::fs::read_dir(path)?;
        let mut texture_files: HashMap<CompactString, Box<[u8]>> = Default::default();

        let mut model_files: HashMap<CompactString, Box<[u8]>> = Default::default();

        let mut blockstate_files: HashMap<CompactString, Box<[u8]>> = Default::default();

        resource_folder.flatten().for_each(|dir_entry| {
            let namespace_path = dir_entry.path();

            event!(Level::INFO, "Loading namespace: {namespace_path:?}");

            event!(Level::INFO, "Loading Textures");
            if let Some(textures) = ResourceLoader::traverse_and_load::<Box<[u8]>>(&namespace_path)
            {
                texture_files.extend(textures);
            }

            event!(Level::INFO, "Loading Models");
            if let Some(models) =
                ResourceLoader::traverse_and_load::<RawBlockModel<'static>>(&namespace_path)
            {
                model_files.extend(models);
            }

            event!(Level::INFO, "Loading Blockstates");
            if let Some(block_states) =
                ResourceLoader::traverse_and_load::<RawBlockVariants<'static>>(&namespace_path)
            {
                blockstate_files.extend(block_states);
            }
        });

        let mut strings = UniqueStrings::new();

        let start = Instant::now();
        let models =
            ResourceLoader::parse_and_intern::<BlockModel<'static>>(&mut strings, model_files);
        let end = Instant::now();
        event!(
            Level::INFO,
            "Time to intern models: {time:?}",
            time = end.duration_since(start)
        );

        let textures = texture_files;

        let start = Instant::now();
        let variants = ResourceLoader::parse_and_intern::<BlockVariants<'static>>(
            &mut strings,
            blockstate_files,
        );
        let end = Instant::now();

        event!(
            Level::INFO,
            "Time to intern variants: {time:?}",
            time = end.duration_since(start)
        );

        Ok(ResourceLoader {
            _strings: strings,
            textures,
            models,
            variants,
        })
    }
    ///Traverses all the folders in a namespace and returns all map of resource locations and file data
    ///for the given resource type T
    #[instrument]
    fn traverse_and_load<'a, 'folder_path, T: Resource<'a>>(
        folder: &'folder_path Path,
    ) -> Option<HashMap<CompactString, Box<[u8]>>> {
        let Some(folder_name) = folder.file_name() else {
            event!(
                Level::WARN,
                "Could not get the namespace component of path: {folder:?}"
            );
            return None;
        };
        let Some(namespace) = folder_name.to_str() else {
            event!(
                Level::WARN,
                "Namespace folder contained invalid utf8: {folder_name:?}"
            );
            return None;
        };

        event!(
            Level::INFO,
            "Attemtping to traverse the namespace: {namespace:?} to find all resources of type {resource_type}\n",resource_type = T::extension()
        );

        let mut resource_type_path = folder.to_path_buf();

        resource_type_path.push(T::folder_name());

        let mut folder_traversal_queue = VecDeque::new();

        let mut file_queue = Vec::new();

        let start = Instant::now();

        if let Ok(entries) = fs::read_dir(&resource_type_path) {
            folder_traversal_queue.extend(
                entries
                    .flatten()
                    .map(|dir_entry| dir_entry.path())
                    .filter(|path| path.is_dir()),
            );
        }

        folder_traversal_queue.push_front(resource_type_path.clone());
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

        event!(
            Level::INFO,
            "Time to traverse folders in namespace: {:?}",
            end.duration_since(start)
        );

        let start = Instant::now();

        event!(
            Level::INFO,
            "Loading all the files in namespace: {namespace}"
        );

        let result = file_queue
            .par_iter()
            .filter_map(|path| {
                let Some(file_stem) = path.file_stem() else {
                    event!(Level::WARN, "File did not have an extension {path:?}");
                    return None;
                };
                let Some(file_name) = file_stem.to_str() else {
                    event!(
                        Level::WARN,
                        "File name contained data that was not valid utf8: {file_stem:?}"
                    );
                    return None;
                };
                let mut resource_path = CompactString::new(namespace);
                resource_path.push(':');

                let Ok(prefix_stripped) = path.strip_prefix(resource_type_path.as_path()) else {
                    event!(
                        Level::DEBUG,
                        "Skipped file in non-resource location{path:?}"
                    );
                    return None;
                };
                let Some(remainder) = prefix_stripped.parent() else {
                    event!(
                        Level::WARN,
                        "Could not strip file name from: {prefix_stripped:?}"
                    );
                    return None;
                };
                let Some(path_str) = remainder.to_str() else {
                    event!(Level::WARN, "Resource location invalid utf8:{remainder:?}");
                    return None;
                };

                resource_path.push_str(path_str);
                if !path_str.is_empty() {
                    resource_path.push('/');
                }
                resource_path.push_str(file_name);

                let Ok(data) = fs::read(path) else {
                    event!(Level::WARN, "Could not load file from path: {path:?}");
                    return None;
                };
                event!(
                    Level::INFO,
                    "Loaded {kind}:{resource_path}",
                    kind = T::folder_name()
                );

                Some((resource_path, data.into_boxed_slice()))
            })
            .collect::<HashMap<_, _>>();

        let end = Instant::now();

        println!("time spent loading: {:?}", end.duration_since(start));

        Some(result)
    }

    fn parse_and_intern<T>(
        strings: &mut UniqueStrings,
        files: HashMap<CompactString, Box<[u8]>>,
    ) -> HashMap<CompactString, T>
    where
        T: InternedResource<'static>,
    {
        let resources: HashMap<CompactString, T> = files
            .into_iter()
            .filter_map(|(resource_path, mut data)| {
                let Ok(resource) = T::View::load(data.as_mut()) else {
                    event!(Level::WARN, "Could not load");
                    return None;
                };
                event!(
                    Level::INFO,
                    "Interning {kind}: {resource_path}",
                    kind = T::View::folder_name()
                );
                let interned = T::View::intern(resource, strings);

                Some((resource_path, interned))
            })
            .collect();
        resources
    }
}

#[derive(Debug, Error)]
pub enum BlockstateLookupError {
    #[error("Mapped state string was not valid: {0}")]
    InvalidStateString(String),
    #[error("No variants found for resource location: {0}")]
    NoVariantsFound(String),
    #[error("Resulting model was empty: mapped_state: {mapped_state}, variants:{variants:?}")]
    EmptyModel {
        mapped_state: String,
        variants: String,
    },
}

impl ResourceLoader {
    pub fn get_blockstates_for_mapped_state<'a>(
        &'a self,
        mapped_state_str: &str,
    ) -> Result<BlockstateType<'a>, BlockstateLookupError> {
        event!(
            Level::INFO,
            "Getting the blockstates for mapped state: {mapped_state_str}"
        );
        let (resource_location, properties_string) =
            mapped_state_str
                .split_once("#")
                .ok_or(BlockstateLookupError::InvalidStateString(
                    mapped_state_str.to_string(),
                ))?;
        event!(
            Level::INFO,
            "Split mapped state into: {resource_location}, {properties_string}"
        );

        let variants =
            self.variants
                .get(resource_location)
                .ok_or(BlockstateLookupError::NoVariantsFound(
                    resource_location.to_string(),
                ))?;

        match variants {
            BlockVariants::Variants(hash_map) => {
                Self::get_model_for_variants(hash_map, properties_string).ok_or(
                    BlockstateLookupError::EmptyModel {
                        mapped_state: mapped_state_str.to_string(),
                        variants: format!("{variants:?}"),
                    },
                )
            }
            BlockVariants::Multipart(cases) => {
                Self::get_models_for_multipart(cases, properties_string).ok_or(
                    BlockstateLookupError::EmptyModel {
                        mapped_state: mapped_state_str.to_string(),
                        variants: format!("{variants:?}"),
                    },
                )
            }
        }
    }

    fn get_model_for_variants<'a>(
        variants: &'a HashMap<&'static str, VariantType<'static>>,
        mut properties_string: &str,
    ) -> Option<BlockstateType<'a>> {
        //Blockstates without properties will have the string "default" in place of a property
        //list, but the variants list in the json will be an empty string
        if properties_string == "default" {
            properties_string = "";
        }

        variants
            .get(properties_string)
            .map(|variant| match variant {
                VariantType::SingleModel(interned_model_properties) => {
                    BlockstateType::SingleModel(std::slice::from_ref(interned_model_properties))
                }
                VariantType::MultiModel(items) => BlockstateType::SingleModel(items.as_slice()),
            })
    }
    #[instrument]
    fn get_models_for_multipart<'a>(
        cases: &'a [Case<'static>],
        variant_string: &str,
    ) -> Option<BlockstateType<'a>> {
        let models = cases
            .iter()
            .filter(|case| case.test_variant_string(variant_string))
            .map(|case| case.get_models())
            .collect::<Vec<_>>();
        if models.is_empty() {
            return None;
        }

        Some(BlockstateType::Multipart(models))
    }
    pub fn get_texture_data(&self, resource_location: &str) -> Option<&[u8]> {
        self.textures.get(resource_location).map(|b| b.as_ref())
    }

    pub fn get_block_model(&self, resource_location: &str) -> Option<&BlockModel<'_>> {
        self.models.get(resource_location)
    }

    pub fn get_variant_data(&self, resource_location: &str) -> Option<&BlockVariants<'_>> {
        self.variants.get(resource_location)
    }
}

#[cfg(test)]
mod tests {
    use tracing_test::traced_test;

    use super::*;

    #[test]
    #[traced_test]
    fn resource_folder() {
        ResourceLoader::load_resource_folder(&PathBuf::from("../resource_pack/assets/")).unwrap();
    }
}
