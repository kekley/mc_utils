use crate::block_model::intern::intern_block_model;
use crate::block_state::borrow::{BlockVariants, Case, ModelResult, VariantType};
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

use compact_str::CompactString;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};

use crate::{
    block_model::serde::RawBlockModel, block_state::serde::RawBlockVariants,
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

trait Resource<'a>: Send + Debug + Sized {
    type Interned;
    fn load(data: &'a mut [u8]) -> Option<Self>;
    fn folder_name() -> &'static str;
    fn extension() -> &'static str;
    fn intern(self, strings: &mut UniqueStrings) -> Self::Interned;
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

    fn intern(self, _strings: &mut UniqueStrings) -> Self::Interned {
        self
    }
}

impl<'a> Resource<'a> for RawBlockVariants<'a> {
    type Interned = BlockVariants<'static>;
    fn load(data: &'a mut [u8]) -> Option<RawBlockVariants<'a>> {
        simd_json::serde::from_slice(data).ok()?
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
    fn load(data: &'a mut [u8]) -> Option<RawBlockModel<'a>> {
        simd_json::serde::from_slice(data).ok().unwrap()
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

pub struct ResourceLoader {
    _strings: UniqueStrings,
    textures: HashMap<CompactString, Box<[u8]>>,
    models: HashMap<CompactString, BlockModel<'static>>,
    variants: HashMap<CompactString, BlockVariants<'static>>,
}

impl ResourceLoader {
    pub fn load_resource_folder(path: &Path) -> Result<ResourceLoader, SpiderEyeError> {
        let resource_folder = std::fs::read_dir(path)?;
        let mut texture_files: HashMap<CompactString, Box<[u8]>> = Default::default();

        let mut model_files: HashMap<CompactString, Box<[u8]>> = Default::default();

        let mut blockstate_files: HashMap<CompactString, Box<[u8]>> = Default::default();

        resource_folder.flatten().for_each(|dir_entry| {
            let namespace_path = dir_entry.path();

            println!("{namespace_path:?}");

            println!("textures");
            if let Some(textures) = ResourceLoader::traverse_and_load::<Box<[u8]>>(&namespace_path)
            {
                texture_files.extend(textures);
            }

            println!("models");
            if let Some(models) =
                ResourceLoader::traverse_and_load::<RawBlockModel<'static>>(&namespace_path)
            {
                model_files.extend(models);
            }

            println!("blockstates");
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
        println!("interned models: {:?}", end.duration_since(start));

        let start = Instant::now();
        let textures = texture_files;
        let end = Instant::now();
        println!("interned textures {:?}", end.duration_since(start));

        let start = Instant::now();
        let variants = ResourceLoader::parse_and_intern::<BlockVariants<'static>>(
            &mut strings,
            blockstate_files,
        );
        let end = Instant::now();
        println!("interned variants {:?}", end.duration_since(start));

        Ok(ResourceLoader {
            _strings: strings,
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

                let data = fs::read(path).ok()?;

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
                let resource = T::View::load(data.as_mut())?;
                println!("interning:{resource_path}");
                let interned = T::View::intern(resource, strings);

                Some((resource_path, interned))
            })
            .collect();
        resources
    }
}

pub struct ModelLookupError {
    kind: ModelLookupErrorKind,
}

enum ModelLookupErrorKind {
    InvalidStateString,
    NoVariantsFound,
    EmptyModel,
}

impl ResourceLoader {
    /*
     *failure cases:
    mapped state string is malformed (no #)
    variants lookup returned none
     * */
    pub fn get_model_for_mapped_state<'a>(
        &'a self,
        mapped_state_str: &str,
    ) -> Option<ModelResult<'a>> {
        println!("{mapped_state_str}");
        let (resource_location, properties_string) = mapped_state_str.split_once("#")?;
        println!("{resource_location}, {properties_string}");

        let variants = if let Some(variant) = self.variants.get(resource_location) {
            variant
        } else {
            println!("variants hashmap lookup failed");
            return None;
        };

        match variants {
            BlockVariants::Variants(hash_map) => {
                Self::get_model_for_variants(hash_map, properties_string)
            }
            BlockVariants::Multipart(cases) => {
                Self::get_models_for_multipart(cases, properties_string)
            }
        }
    }

    fn get_model_for_variants<'a>(
        variants: &'a HashMap<&'static str, VariantType<'static>>,
        mut properties_string: &str,
    ) -> Option<ModelResult<'a>> {
        if properties_string == "default" {
            properties_string = "";
        }
        variants
            .get(properties_string)
            .map(|variant| match variant {
                VariantType::SingleModel(interned_model_properties) => {
                    ModelResult::SingleModel(std::slice::from_ref(interned_model_properties))
                }
                VariantType::MultiModel(items) => ModelResult::SingleModel(items.as_slice()),
            })
    }
    fn get_models_for_multipart<'a>(
        cases: &'a [Case<'static>],
        variant_string: &str,
    ) -> Option<ModelResult<'a>> {
        let models = cases
            .iter()
            .filter(|case| case.test_variant_string(variant_string))
            .map(|case| case.get_models())
            .collect::<Vec<_>>();
        if models.is_empty() {
            eprintln!("no models from multipart");
            return None;
        }

        Some(ModelResult::Multipart(models))
    }
    pub fn get_texture_data(&self, resource_location: &str) -> Option<&[u8]> {
        self.textures.get(resource_location).map(|b| b.as_ref())
    }

    pub fn get_block_model(&self, resource_location: &str) -> Option<&BlockModel<'_>> {
        println!("looking for model at: {resource_location}");
        self.models.get(resource_location)
    }

    pub fn get_variant_data(&self, resource_location: &str) -> Option<&BlockVariants<'_>> {
        self.variants.get(resource_location)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn resource_folder() {
        let a =
            ResourceLoader::load_resource_folder(&PathBuf::from("./test_assets/assets/")).unwrap();
        let models = a.models;

        println!("{:?}", models);
    }
}
