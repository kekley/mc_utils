use bumpalo::Bump;
use bytes::Bytes;

use crate::{
    block::Block,  chunk::Chunk, loaded_world::World, nbt_compound::NBTCompound, variant::ModelVariant
};

use super::{block_models::ASSET_PATH, resource::ModelVariants};
pub type BlockStateIndex = usize;
pub struct MCResourceLoader {
    arena: Bump,
}

impl MCResourceLoader {
    pub fn load_block_states_interned(&self, block_name: &str) -> Option<ModelVariants> {
        let split = block_name
            .split_once(":")
            .unwrap_or(("minecraft", block_name));
        let namespace = split.0;
        let block_name_split = split.1;
        let mut path = String::new();
        path.push_str(&ASSET_PATH);
        path.push_str(namespace);
        path.push_str("/");
        path.push_str("blockstates/");

        path.push_str(block_name_split);
        path.push_str(".json");

        let new_state = ModelVariants::load_from_json(&path, &self.arena);
    }
    pub fn load_variants_for(
        &self,
        block: &Block,
        block_states: &ModelVariants,
    ) -> Vec<ModelVariant> {
        let variants = match block_states {
            ModelVariants::MultipartVariant(multipart) => {
                multipart.load_models(&block.properties, &self.rodeo)
            }
            ModelVariants::StandardVariant(variants) => {
                variants.get_model_variants(&block.properties)
            }
        };
        variants
    }
    pub fn open_world(&self, region_folder: &str) -> Result<World> {
        World::new(region_folder, &self.rodeo)
    }

    pub fn nbt_from_bytes(&self, bytes: &mut Bytes) -> anyhow::Result<NBTCompound> {
        NBTCompound::new_from_bytes(bytes, &self.rodeo)
    }

    pub(crate) fn chunk_from_nbt(&self, chunk_nbt: NBTCompound) -> Option<Chunk> {
        Some({
            let binding = chunk_nbt
                .get_tag("")
                .expect_tag("Chunks must start with an empty name compound tag")?;
            let chunk = binding.get_compound();
            let data_version = chunk
                .get_tag("DataVersion")
                .expect("Not a Chunk NBT")
                .get_int();
            let xpos = chunk.get_tag("xPos").expect("Not a Chunk NBT").get_int();
            let zpos = chunk.get_tag("zPos").expect("Not a Chunk NBT").get_int();

            let sections = chunk.get_tag("sections").unwrap().get_list();

            let section_array: Vec<ChunkSection> = sections
                .iter()
                .filter_map(|section| {
                    let section_compound = section.get_compound();
                    let section = ChunkSection::from_compound_internal(&section_compound);
                    if section.ypos >= -4 {
                        Some(section)
                    } else {
                        None
                    }
                })
                .collect::<Vec<_>>();

            let lowest_section = section_array
                .iter()
                .min_by_key(|s| s.ypos)
                .expect("empty section array");

            let min = lowest_section.ypos as isize;
            let max = section_array
                .iter()
                .max_by_key(|s| s.ypos)
                .map(|s| s.ypos)
                .unwrap() as isize;
            let mut sparse_sections = vec![None; (1 + max - min) as usize];

            for (i, sec) in section_array.iter().enumerate() {
                let sec_index = (sec.ypos as isize - min) as usize;

                sparse_sections[sec_index] = Some(i);
            }

            let sec_tower = SectionTower {
                sections: section_array,
                map: sparse_sections,
                y_min: 16 * min,
                y_max: 16 * (max + 1),
            };

            let coords = ChunkCoords::new(xpos.into(), zpos.into());

            Chunk<'a> {
                coords,
                data_version,
                sections: sec_tower,
            }
        })
    }

    pub fn new() -> Self {
        Self { arena: Bump::new() }
    }
    pub fn get_texture_path(&self, resource_path: &str) -> String {
        let (namespace, remaining_str) = resource_path
            .split_once(":")
            .unwrap_or(("minecraft", resource_path));
        //dbg!(namespace, remaining_str);

        let (model_type, remaining_str) = remaining_str
            .split_once("/")
            .expect("invalid path for parent");
        //dbg!(model_type, remaining_str);
        String::from(
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
    pub fn resolve_spur(&self, spur: &Spur) -> &str {
        self.rodeo.resolve(spur)
    }
}
