use chrono::Local;
use indicatif::{ProgressBar, ProgressStyle};
use reqwest::get;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self, File};
use std::io::{Read, Write};
use std::sync::Arc;
use std::time::Duration;
use tokio::io::AsyncWriteExt;
use tokio::sync::Semaphore;
use url::Url;
use walkdir::WalkDir;
use zip::ZipWriter;
use zip::write::FileOptions;

#[derive(Debug, Deserialize)]
struct PackageLock {
    packages: HashMap<String, PackageInfo>,
}

#[derive(Debug, Deserialize)]
struct PackageInfo {
    resolved: Option<String>,
}

fn get_timestamped_dir() -> String {
    let now = Local::now();
    format!("packages_{}", now.format("%Y%m%d_%H%M%S"))
}

fn ensure_output_dir(dir_name: &str) -> Result<(), Box<dyn std::error::Error>> {
    fs::create_dir_all(dir_name)?;
    Ok(())
}

fn filename_from_url(url: &str) -> Option<String> {
    let parsed = Url::parse(url).ok()?;
    parsed.path_segments()?.last().map(|s| s.to_string())
}

fn read_package_lock(path: &str) -> Result<PackageLock, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let lockfile: PackageLock = serde_json::from_str(&content)?;
    Ok(lockfile)
}

async fn download_package(url: &str, outputdir: &str) -> Result<(), Box<dyn std::error::Error>> {
    let filename = filename_from_url(url).ok_or("Impossible d'extraire le nom du fichier")?;

    let response = get(url).await?;

    let path = format!("{}/{}", outputdir, &filename);

    let mut file = tokio::fs::File::create(&path).await?;
    let bytes = response.bytes().await?;
    file.write_all(&bytes).await?;

    Ok(())
}

fn zip_dir(dir_path: &str, zip_path: &str) -> Result<(), Box<dyn std::error::Error>> {
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

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lock = read_package_lock("package-lock.json")?;

    let dir_name = get_timestamped_dir();
    ensure_output_dir(&dir_name)?;

    let thread_count = 4;
    let mut handles = vec![];

    let urls: Vec<String> = lock
        .packages
        .values()
        .filter_map(|pkg| pkg.resolved.clone())
        .collect();

    let pb = ProgressBar::new(urls.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    let semaphore = Arc::new(Semaphore::new(thread_count));

    for url in urls {
        let dir_name = dir_name.clone();
        let pb = pb.clone();
        let semaphore = semaphore.clone();
        let handle = tokio::spawn(async move {
            let _ = semaphore.acquire().await.unwrap();
            if let Err(e) = download_package(&url, &dir_name).await {
                eprintln!("Failed to download {}: {}", url, e);
            }
            pb.inc(1);
        });
        handles.push(handle);
    }

    for handle in handles {
        handle.await?;
    }
    pb.finish_with_message("Téléchargement terminé");

    let zip_name = format!("{}.zip", &dir_name);

    zip_dir(&dir_name, &zip_name)?;

    fs::remove_dir_all(&dir_name)?;
    Ok(())
}
