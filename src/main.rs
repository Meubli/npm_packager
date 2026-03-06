use reqwest::{Client, get};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use tokio::fs::File;
use tokio::io::{AsyncWriteExt, copy};
use url::Url;

#[derive(Debug, Deserialize)]
struct PackageLock {
    packages: HashMap<String, PackageInfo>,
}

#[derive(Debug, Deserialize)]
struct PackageInfo {
    resolved: Option<String>,
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

async fn download_package(url: &str) -> Result<(), Box<dyn std::error::Error>> {
    let filename = filename_from_url(url).ok_or("Impossible d'extraire le nom du fichier")?;

    let response = get(url).await?;

    let mut file = File::create(&filename).await?;
    let bytes = response.bytes().await?;
    file.write_all(&bytes).await?;

    println!("Téléchargé: {}", filename);

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let lock = read_package_lock("package-lock.json")?;

    let mut handles = vec![];

    for pkg in lock.packages.values() {
        if let Some(url) = &pkg.resolved {
            let url = url.clone();
            let handle = tokio::spawn(async move {
                if let Err(e) = download_package(&url).await {
                    eprintln!("Failed to download {}: {}", url, e);
                }
            });
            handles.push(handle);
        }
    }

    for handle in handles {
        handle.await?;
    }
    Ok(())
}
