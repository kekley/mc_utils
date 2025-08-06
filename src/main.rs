use std::{error::Error, time::Instant};

use spider_eye::{
    borrow::nbt_compound::RootNBTCompound, chunk::borrow::Chunk, region::borrow::Region,
    resource_loader::load_folder, section::borrow::Section,
};

pub fn main() -> Result<(), Box<dyn Error>> {
    tracing_subscriber::fmt::init();

    let mut chunk_data_vec = vec![];
    let region = Region::load_from_file("./test_world/r.2.-1.mca")?;
    for z in 0..32 {
        for x in 0..32 {
            let chunk_data = region.load_chunk_data(x, z);
            if let Some(chunk_data) = chunk_data {
                chunk_data_vec.push(chunk_data);
            }
        }
    }
    let mut chunks_vec = vec![];
    for chunk_data in &chunk_data_vec {
        let nbt = RootNBTCompound::from_bytes(chunk_data);

        if let Ok(root_nbt) = nbt {
            let chunk = Chunk::from_compound(root_nbt);

            if let Some(chunk) = chunk {
                chunks_vec.push(chunk);
            }
        }
    }

    let start = Instant::now();
    for chunk in &chunks_vec {
        let x = chunk.get_x();
        let z = chunk.get_z();

        let sections = chunk.get_sections().unwrap();
        let mut num_sections = 0;
        sections
            .iter_sections()
            .for_each(|section: Section<'_, '_>| {
                num_sections += 1;
                let y = section.get_lowest_y();
                for (index, block) in section.iter_blocks().enumerate() {
                    let y = y + (index as isize / 16 / 16) % 16;
                    let z = z + (index as isize / 16) % 16;
                    let x = x + (index as isize % 16);

                    std::hint::black_box(block.get_name());
                }
            });
    }

    let end = Instant::now();

    dbg!(end.duration_since(start));

    println!("{}", (size_of::<Chunk<'_>>() * chunks_vec.len()) / 1000);
    let mut sum = 0;
    chunk_data_vec.iter().for_each(|f| sum += f.len());
    println!("{}", sum / 1000);

    let _ = load_folder("./test_assets/assets/");
    Ok(())
}
