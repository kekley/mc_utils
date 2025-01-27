use spider_eye::{resource_loader::ResourceLoader, SpiderEyeError};

extern crate spider_eye;

fn main() -> Result<(), SpiderEyeError> {
    let loader = ResourceLoader::new();
    let a = loader.load_model("./test_assets/assets/minecraft/models/block/acacia_button.json");
    dbg!(a);
    Ok(())
}
