use std::fs;
use std::fs::File;
use std::io::Write;
use std::path::Path;
use include_dir::include_dir;

/// Write out all assets that should be included.
pub fn include_static_assets(root_path: &Path) -> Result<(), String> {
    let static_assets_dir_path = root_path.join("assets/static");
    let assets = include_dir!("assets-include");
    fs::create_dir_all(&static_assets_dir_path);
    for asset in assets.files() {
        let asset_path = static_assets_dir_path.join(asset.path());
        let mut file = File::create(&asset_path).unwrap();
        if let Err(err) = file.write_all(asset.contents()) {
            return Err(format!("Error writing asset file {}.", asset_path.to_str().unwrap()));
        }
    }
    Ok(())
}

pub fn include_assets(root_path: &Path) -> Result<(), String> {
    Ok(()) // TODO
}
