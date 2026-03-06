use clap::Parser;
use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::{self};
use std::time::Duration;

use crate::download::download_package_with_retry;
use crate::system::{ensure_output_dir, get_timestamped_dir, zip_dir};

mod download;
mod system;

#[derive(Debug, Deserialize)]
struct PackageLock {
    packages: HashMap<String, PackageInfo>,
}

#[derive(Debug, Deserialize)]
struct PackageInfo {
    resolved: Option<String>,
}

#[derive(Parser, Debug)]
#[command(
    author,
    version,
    about = "Télécharge et empaquette les dépendances npm"
)]
struct Args {
    /// Chemin vers le fichier package-lock.json
    #[arg(short, long, default_value = "package-lock.json")]
    package_lock: String,

    /// Nombre de téléchargements concurrents
    #[arg(short, long, default_value = "100")]
    concurrent: usize,

    /// Nombre maximal de tentatives pour chaque téléchargement
    #[arg(short, long, default_value = "4")]
    max_retries: u16,
}

fn read_package_lock(path: &str) -> Result<PackageLock, Box<dyn std::error::Error>> {
    let content = fs::read_to_string(path)?;
    let lockfile: PackageLock = serde_json::from_str(&content)?;
    Ok(lockfile)
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();

    let lock = read_package_lock(&args.package_lock)?;

    let dir_name = get_timestamped_dir();
    ensure_output_dir(&dir_name)?;

    let concurrent_downloads = args.concurrent;
    let max_retries: u16 = args.max_retries;

    let urls: Vec<String> = lock
        .packages
        .values()
        .filter_map(|pkg| pkg.resolved.clone())
        .collect();
    let failed_urls = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));

    let pb = ProgressBar::new(urls.len() as u64);
    pb.set_style(
        ProgressStyle::with_template(
            "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
        )
        .unwrap()
        .progress_chars("##-"),
    );

    // Créer un stream de futures et les exécuter avec buffered()
    let futures = stream::iter(urls)
        .map(|url| {
            let dir_name = dir_name.clone();
            let pb = pb.clone();
            let failed_urls = failed_urls.clone();

            async move {
                match download_package_with_retry(&url, &dir_name, max_retries).await {
                    Ok(_) => {
                        // eprintln!("✓ {}", url);
                    }
                    Err(e) => {
                        eprintln!("✗ {}: {}", url, e);
                        failed_urls.lock().await.push(url.clone());
                    }
                }
                pb.inc(1);
            }
        })
        .buffered(concurrent_downloads);

    futures.collect::<()>().await;

    println!("Téléchargement terminé.");

    // Afficher les URL qui ont échoué
    let failed = failed_urls.lock().await;
    if !failed.is_empty() {
        eprintln!(
            "\n{} package(s) n'ont pas pu être téléchargés:",
            failed.len()
        );
        for url in failed.iter() {
            eprintln!("  - {}", url);
        }
        let failed_list = failed.join("\n");
        fs::write(format!("{}/failed_packages.txt", &dir_name), failed_list)?;
    }

    let spinner = ProgressBar::new_spinner();
    spinner.enable_steady_tick(Duration::from_millis(100));
    spinner.set_message("Compression des packages...");

    let zip_name = format!("{}.zip", &dir_name);

    zip_dir(&dir_name, &zip_name)?;

    fs::remove_dir_all(&dir_name)?;

    spinner.finish_with_message("Terminé !");
    Ok(())
}
