use clap::Parser;
use tracing_subscriber::EnvFilter;

mod config;
mod download;
mod error;
mod packager;
mod system;

use config::Args;
use config::PackagerConfig;
use packager::Packager;

#[tokio::main]
async fn main() {
    // Initialisation du logging avec tracing
    init_logging();

    // Parsing des arguments CLI
    let args = Args::parse();

    // Création de la configuration
    let config = PackagerConfig::from_args(args);

    // Création du packager
    let packager = Packager::new(config);

    // Exécution
    match packager.run().await {
        Ok(zip_path) => {
            println!("✓ Succès ! Archive créée: {}", zip_path);
        }
        Err(e) => {
            eprintln!("✗ Erreur: {}", e);
            std::process::exit(1);
        }
    }
}

/// Initialise le système de logging avec tracing
fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::fmt().with_env_filter(env_filter).init();
}
