use clap::Parser;
use serde::Deserialize;
use std::collections::HashMap;
use std::path::PathBuf;

/// Configuration de l'application, parsée depuis les arguments CLI
#[derive(Parser, Debug, Clone)]
#[command(
    author,
    version,
    about = "Télécharge et empaquette les dépendances npm"
)]
pub struct Args {
    /// Chemin vers le fichier package-lock.json
    #[arg(short, long, default_value = "package-lock.json")]
    pub package_lock: PathBuf,

    /// Nombre de téléchargements concurrents
    #[arg(short, long, default_value = "100")]
    pub concurrent: usize,

    /// Nombre maximal de tentatives pour chaque téléchargement
    #[arg(short, long, default_value = "4")]
    pub max_retries: u16,

    /// Répertoire de sortie (optionnel, utilise un répertoire timestampé par défaut)
    #[arg(short, long)]
    pub output_dir: Option<PathBuf>,
}

/// Structure représentant le fichier package-lock.json
#[derive(Debug, Deserialize)]
pub struct PackageLock {
    pub packages: HashMap<String, PackageInfo>,
}

/// Informations sur un package individuel dans package-lock.json
#[derive(Debug, Deserialize)]
pub struct PackageInfo {
    pub resolved: Option<String>,
    pub version: Option<String>,
    pub integrity: Option<String>,
}

/// Configuration du packager (version plus lisible et typée)
#[derive(Debug, Clone)]
pub struct PackagerConfig {
    pub package_lock_path: PathBuf,
    pub output_dir: Option<PathBuf>,
    pub concurrent_downloads: usize,
    pub max_retries: u16,
}

impl PackagerConfig {
    /// Crée une configuration à partir des arguments CLI
    pub fn from_args(args: Args) -> Self {
        PackagerConfig {
            package_lock_path: args.package_lock,
            output_dir: args.output_dir,
            concurrent_downloads: args.concurrent,
            max_retries: args.max_retries,
        }
    }

    /// Valide la configuration
    pub fn validate(&self) -> crate::error::PackagerResult<()> {
        if !self.package_lock_path.exists() {
            return Err(crate::error::PackagerError::Config {
                reason: format!(
                    "Le fichier package-lock.json n'existe pas: {}",
                    self.package_lock_path.display()
                ),
            });
        }

        if self.concurrent_downloads == 0 {
            return Err(crate::error::PackagerError::Config {
                reason: "Le nombre de téléchargements concurrents doit être > 0".to_string(),
            });
        }

        if self.max_retries == 0 {
            return Err(crate::error::PackagerError::Config {
                reason: "Le nombre maximal de tentatives doit être > 0".to_string(),
            });
        }

        Ok(())
    }
}
