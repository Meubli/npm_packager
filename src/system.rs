use std::{
    fs::{self, File},
    io::{Read, Write},
};

use chrono::Local;
use walkdir::WalkDir;
use zip::{ZipWriter, write::FileOptions};

pub fn zip_dir(dir_path: &str, zip_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let file = File::create(zip_path)?;
    let mut zip = ZipWriter::new(file);

    for entry in WalkDir::new(dir_path) {
        let entry = entry?;
        let path = entry.path();
        let name = path
            .strip_prefix(dir_path)?
            .to_str()
            .ok_or("Invalid path")?
            .to_owned();

        if path.is_file() {
            zip.start_file(name, FileOptions::default())?;
            let mut f = File::open(path)?;
            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer)?;
            zip.write_all(&buffer)?;
        }
    }

    zip.finish()?;
    Ok(())
}

pub fn get_timestamped_dir() -> String {
    let now = Local::now();
    format!("packages_{}", now.format("%Y%m%d_%H%M%S"))
}

pub fn ensure_output_dir(dir_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir_name)?;
    Ok(())
}
