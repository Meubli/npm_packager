use std::path::Path;
use std::time::Duration;

use base64::engine::{general_purpose, Engine as _};
use reqwest::get;
use sha2::{Digest, Sha512};
use tokio::{io::AsyncWriteExt, time::sleep};

use crate::error::{PackagerError, PackagerResult};

#[derive(Debug, Clone)]
pub struct Package {
    pub url: String,
    pub name: String,
    pub version: String,
    pub integrity: String,
}

impl Package {
    pub fn new(url: String, name: String, version: String, integrity: String) -> Self {
        Self {
            url,
            name,
            version,
            integrity,
        }
    }

    fn filename(&self) -> String {
        format!("{}-{}.tgz", self.name.replace('/', "-"), self.version)
    }
}

pub async fn download_package_with_retry(
    package: &Package,
    output_dir: &Path,
    max_retry: u16,
) -> PackagerResult<()> {
    let mut retry = 0;
    let mut delay = Duration::from_millis(500);

    loop {
        match download_package(package, output_dir).await {
            Ok(_) => return Ok(()),
            Err(PackagerError::Integrity { reason, .. }) => {
                return Err(PackagerError::Integrity {
                    url: package.url.clone(),
                    reason,
                });
            }
            Err(e) => {
                retry += 1;

                if retry >= max_retry {
                    return Err(PackagerError::Download {
                        url: package.url.clone(),
                        reason: format!("Échec après {} tentatives: {}", max_retry, e),
                    });
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

async fn download_package(package: &Package, output_dir: &Path) -> PackagerResult<()> {
    let filename = package.filename();

    let response = get(&package.url)
        .await
        .map_err(|e| PackagerError::Download {
            url: package.url.clone(),
            reason: e.to_string(),
        })?;

    if !response.status().is_success() {
        return Err(PackagerError::Download {
            url: package.url.clone(),
            reason: format!("Statut HTTP: {}", response.status()),
        });
    }

    let path = output_dir.join(&filename);

    let mut file = tokio::fs::File::create(&path)
        .await
        .map_err(|e| PackagerError::Io {
            path: path.clone(),
            reason: e.to_string(),
        })?;

    let bytes = response
        .bytes()
        .await
        .map_err(|e| PackagerError::Download {
            url: package.url.clone(),
            reason: e.to_string(),
        })?;

    verify_integrity(&bytes, &package.integrity, &package.url)?;

    file.write_all(&bytes)
        .await
        .map_err(|e| PackagerError::Io {
            path,
            reason: e.to_string(),
        })?;

    Ok(())
}

/// Vérifie l'intégrité du fichier en comparant le hash SHA512
/// Format attendu: "sha512-{hash_en_base64}"
fn verify_integrity(bytes: &[u8], integrity_string: &str, url: &str) -> PackagerResult<()> {
    let parts: Vec<&str> = integrity_string.splitn(2, '-').collect();

    if parts.len() != 2 {
        return Err(PackagerError::Integrity {
            url: url.to_string(),
            reason: "Format d'intégrité invalide".to_string(),
        });
    }

    let algorithm = parts[0].to_lowercase();
    let expected_hash = parts[1];

    match algorithm.as_str() {
        "sha512" => {
            // Calculer le hash SHA512 des bytes
            let mut hasher = Sha512::new();
            hasher.update(bytes);
            let hash_result = hasher.finalize();

            // Convertir en base64 (npm utilise le base64)
            let computed_base64 = general_purpose::STANDARD.encode(hash_result);

            if computed_base64 == expected_hash {
                Ok(())
            } else {
                Err(PackagerError::Integrity {
                    url: url.to_string(),
                    reason: format!(
                        "SHA512 invalide. Attendu: {}, Obtenu: {}",
                        expected_hash, computed_base64
                    ),
                })
            }
        }
        other => Err(PackagerError::Integrity {
            url: url.to_string(),
            reason: format!("Algorithme non supporté: {}", other),
        }),
    }
}
