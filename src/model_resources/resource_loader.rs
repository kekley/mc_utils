use crate::error::spider_eye_error::SpiderEyeError;

const ASSET_FOLDER_NAMES: [&str; 3] = ["blockstates", "models", "textures"];

pub fn load_folder(path: &str) -> Result<LoadedResources, SpiderEyeError> {
    let asset_folder = std::fs::read_dir(path)?;

    let namespaces: Vec<_> = asset_folder
        .into_iter()
        .filter_map(|dir| {
            let entry = dir.ok()?;

            let path = entry.path();

            if !path.is_dir() {
                return None;
            }

            let inner_folders = std::fs::read_dir(&path).ok()?;

            if inner_folders
                .into_iter()
                .filter(|folder| {
                    if let Ok(dir_entry) = folder {
                        ASSET_FOLDER_NAMES
                            .iter()
                            .any(|folder_name| dir_entry.path().ends_with(folder_name))
                    } else {
                        false
                    }
                })
                .count()
                == 3
            {
                Some(path)
            } else {
                None
            }
        })
        .collect();

    println!("{namespaces:?}");

    Ok(todo!())
}

pub struct LoadedResources {}
