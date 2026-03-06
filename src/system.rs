use std::fs;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};

use chrono::Local;
use walkdir::WalkDir;
use zip::{ZipWriter, write::FileOptions};

use crate::error::{PackagerError, PackagerResult};

/// Compresse un répertoire en fichier ZIP
pub fn zip_dir(dir_path: &Path, zip_path: &str) -> PackagerResult<()> {
    let file = fs::File::create(zip_path).map_err(|e| PackagerError::Io {
        path: PathBuf::from(zip_path),
        reason: format!("Impossible de créer le fichier ZIP: {}", e),
    })?;

    let mut zip = ZipWriter::new(file);

    for entry in WalkDir::new(dir_path) {
        let entry = entry.map_err(|e| PackagerError::Compression {
            path: dir_path.to_path_buf(),
            reason: format!("Erreur lors du parcours du répertoire: {}", e),
        })?;

        let path = entry.path();
        let name = path
            .strip_prefix(dir_path)
            .map_err(|e| PackagerError::Compression {
                path: dir_path.to_path_buf(),
                reason: format!("Erreur lors du calcul du chemin relatif: {}", e),
            })?
            .to_str()
            .ok_or(PackagerError::Compression {
                path: dir_path.to_path_buf(),
                reason: "Chemin contient des caractères invalides".to_string(),
            })?
            .to_owned();

        if path.is_file() {
            zip.start_file(name, FileOptions::default()).map_err(|e| {
                PackagerError::Compression {
                    path: dir_path.to_path_buf(),
                    reason: format!("Erreur lors de l'ajout d'un fichier au ZIP: {}", e),
                }
            })?;

            let mut f = fs::File::open(path).map_err(|e| PackagerError::Io {
                path: path.to_path_buf(),
                reason: format!("Impossible d'ouvrir le fichier: {}", e),
            })?;

            let mut buffer = Vec::new();
            f.read_to_end(&mut buffer).map_err(|e| PackagerError::Io {
                path: path.to_path_buf(),
                reason: format!("Erreur de lecture du fichier: {}", e),
            })?;

            zip.write_all(&buffer)
                .map_err(|e| PackagerError::Compression {
                    path: dir_path.to_path_buf(),
                    reason: format!("Erreur lors de l'écriture au ZIP: {}", e),
                })?;
        }
    }

    zip.finish().map_err(|e| PackagerError::Compression {
        path: dir_path.to_path_buf(),
        reason: format!("Erreur lors de la finalisation du ZIP: {}", e),
    })?;

    Ok(())
}

/// Génère un nom de répertoire avec timestamp
pub fn get_timestamped_dir() -> PathBuf {
    let now = Local::now();
    let dirname = format!("packages_{}", now.format("%Y%m%d_%H%M%S"));
    PathBuf::from(dirname)
}

/// Crée le répertoire de sortie s'il n'existe pas
pub fn ensure_output_dir(dir_path: &Path) -> PackagerResult<()> {
    fs::create_dir_all(dir_path).map_err(|e| PackagerError::Io {
        path: dir_path.to_path_buf(),
        reason: format!("Impossible de créer le répertoire: {}", e),
    })?;
    Ok(())
}
