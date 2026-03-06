use std::time::Duration;

use reqwest::get;
use tokio::{io::AsyncWriteExt, time::sleep};
use url::Url;

#[derive(Debug, Clone)]
pub struct Package {
    pub url: String,
    pub name: String,
    pub version: String,
}

impl Package {
    pub fn new(url: String, name: String, version: String) -> Self {
        Self { url, name, version }
    }

    fn filename(&self) -> String {
        format!("{}-{}", self.name.replace("/", "-"), self.version)
    }
}

pub async fn download_package_with_retry(
    package: &Package,
    outputdir: &str,
    max_retry: u16,
) -> Result<(), String> {
    let mut retry = 0;
    let mut delay = Duration::from_millis(500);

    loop {
        match download_package(package, outputdir).await {
            Ok(_) => return Ok(()),
            Err(e) => {
                retry += 1;

                if retry >= max_retry {
                    return Err(format!(
                        "Échec après {} tentatives pour {}: {}",
                        max_retry, package.name, e
                    ));
                }

                eprintln!(
                    "Tentative {} échouée pour {} ({}): {}. Réessai dans {:?}...",
                    retry, package.name, package.url, e, delay
                );
                sleep(delay).await;
                delay = Duration::from_millis(delay.as_millis() as u64 * 2);
            }
        }
    }
}

async fn download_package(package: &Package, outputdir: &str) -> Result<(), String> {
    let filename = package.filename();

    let response = get(&package.url).await.map_err(|e| e.to_string())?;

    if !response.status().is_success() {
        return Err(format!("Statut HTTP: {}", response.status()));
    }

    let path = format!("{}/{}", outputdir, &filename);

    let mut file = tokio::fs::File::create(&path)
        .await
        .map_err(|e| e.to_string())?;
    let bytes = response.bytes().await.map_err(|e| e.to_string())?;
    file.write_all(&bytes).await.map_err(|e| e.to_string())?;

    Ok(())
}
