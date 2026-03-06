use futures::stream::{self, StreamExt};
use indicatif::{ProgressBar, ProgressStyle};
use std::fs;
use std::sync::Arc;
use std::time::Duration;

use crate::config::{PackageLock, PackagerConfig};
use crate::download::{Package, download_package_with_retry};
use crate::error::{PackagerError, PackagerResult};
use crate::system::{ensure_output_dir, get_timestamped_dir, zip_dir};

/// Orchestrateur principal pour le téléchargement et l'empaquetage
pub struct Packager {
    config: PackagerConfig,
}

impl Packager {
    /// Crée une nouvelle instance de Packager
    pub fn new(config: PackagerConfig) -> Self {
        Packager { config }
    }

    /// Lance l'orchestration complète du téléchargement et l'empaquetage
    pub async fn run(&self) -> PackagerResult<String> {
        // Validation de la configuration
        self.config.validate()?;

        // Lecture du fichier package-lock.json
        let lockfile = self.read_package_lock()?;

        // Détermination du répertoire de sortie
        let output_dir = if let Some(dir) = &self.config.output_dir {
            dir.clone()
        } else {
            get_timestamped_dir()
        };

        // Création du répertoire de sortie
        ensure_output_dir(&output_dir)?;

        // Extraction des packages valides
        let packages = self.extract_packages(&lockfile);
        let total_packages = packages.len();

        if total_packages == 0 {
            return Err(PackagerError::Config {
                reason: "Aucun package trouvé dans package-lock.json".to_string(),
            });
        }

        println!("Téléchargement de {} packages en cours...", total_packages);

        // Téléchargement avec gestion des erreurs
        let failed_packages = self.download_packages(packages, &output_dir).await;

        // Rapport sur les échecs
        if !failed_packages.is_empty() {
            self.report_failed_packages(&output_dir, &failed_packages)?;
        }

        // Compression avec spinner
        let spinner = ProgressBar::new_spinner();
        spinner.enable_steady_tick(Duration::from_millis(100));
        spinner.set_message("Compression des packages...");

        let zip_path = format!("{}.zip", output_dir.display());
        zip_dir(&output_dir, &zip_path)?;

        spinner.finish_with_message("Compression terminée !");

        // Nettoyage
        fs::remove_dir_all(&output_dir).map_err(|e| PackagerError::Io {
            path: output_dir.clone(),
            reason: format!("Impossible de supprimer le répertoire temporaire: {}", e),
        })?;

        Ok(zip_path)
    }

    /// Lit et parse le fichier package-lock.json
    fn read_package_lock(&self) -> PackagerResult<PackageLock> {
        let content =
            fs::read_to_string(&self.config.package_lock_path).map_err(|e| PackagerError::Io {
                path: self.config.package_lock_path.clone(),
                reason: format!("Impossible de lire le fichier: {}", e),
            })?;

        serde_json::from_str(&content).map_err(|e| PackagerError::Parse {
            reason: e.to_string(),
        })
    }

    /// Extrait les packages valides du lockfile
    fn extract_packages(&self, lockfile: &PackageLock) -> Vec<Package> {
        lockfile
            .packages
            .iter()
            .filter_map(|(name, pkg_info)| {
                let url = pkg_info.resolved.as_ref()?.clone();
                let version = pkg_info.version.as_ref()?.clone();
                let integrity = pkg_info.integrity.as_ref()?.clone();
                let name = name.replace("node_modules/", "");

                Some(Package::new(url, name, version, integrity))
            })
            .collect()
    }

    /// Télécharge tous les packages avec gestion de la concurrence
    async fn download_packages(
        &self,
        packages: Vec<Package>,
        output_dir: &std::path::Path,
    ) -> Vec<Package> {
        let total = packages.len();
        let failed_packages = Arc::new(tokio::sync::Mutex::new(Vec::new()));

        let pb = ProgressBar::new(total as u64);
        pb.set_style(
            ProgressStyle::with_template(
                "[{elapsed_precise}] [{bar:40.cyan/blue}] {pos}/{len} ({eta})",
            )
            .unwrap()
            .progress_chars("##-"),
        );

        let futures = stream::iter(packages)
            .map(|package| {
                let output_dir = output_dir.to_path_buf();
                let pb = pb.clone();
                let failed_packages = failed_packages.clone();
                let max_retries = self.config.max_retries;

                async move {
                    match download_package_with_retry(&package, &output_dir, max_retries).await {
                        Ok(_) => {
                            // Silence - le progress bar suffit
                        }
                        Err(e) => {
                            eprintln!("✗ {}-{}: {}", package.name, package.version, e);
                            failed_packages.lock().await.push(package);
                        }
                    }
                    pb.inc(1);
                }
            })
            .buffered(self.config.concurrent_downloads);

        futures.collect::<()>().await;
        pb.finish();

        Arc::try_unwrap(failed_packages).unwrap().into_inner()
    }

    /// Génère un rapport sur les packages ayant échoué
    fn report_failed_packages(
        &self,
        output_dir: &std::path::Path,
        failed_packages: &[Package],
    ) -> PackagerResult<()> {
        eprintln!(
            "\n{} package(s) n'ont pas pu être téléchargés:",
            failed_packages.len()
        );
        for pkg in failed_packages {
            eprintln!("  - {} ({})", pkg.name, pkg.version);
        }

        let failed_list = failed_packages
            .iter()
            .map(|pkg| format!("{} ({}): {}", pkg.name, pkg.version, pkg.url))
            .collect::<Vec<_>>()
            .join("\n");

        let failed_file = output_dir.join("failed_packages.txt");
        fs::write(&failed_file, failed_list).map_err(|e| PackagerError::Io {
            path: failed_file,
            reason: format!("Impossible d'écrire le fichier de packages échoués: {}", e),
        })?;

        Ok(())
    }
}
