# Architecture du npm_packager

## Vue d'ensemble

Le projet suit une architecture modulaire avec séparation claire des responsabilités. Chaque module a un rôle spécifique et dépend d'autres modules de manière prévisible.

```
┌─────────────────────────────────────┐
│        main.rs                      │ Point d'entrée CLI
│  (CLI parsing + orchestration)      │
└────────┬────────────────────────────┘
         │
         ├──────────────────────────────────────────┐
         │                                          │
    ┌────▼─────────┐                        ┌──────▼──────────┐
    │ config.rs    │                        │  packager.rs    │
    │ (Config)     │────────────────────►   │ (Orchestration) │
    └──────────────┘                        └──────┬──────────┘
                                                   │
                                    ┌──────────────┼──────────────┐
                                    │              │              │
                            ┌───────▼──┐  ┌───────▼─────┐ ┌────▼────┐
                            │download   │  │ system.rs   │ │error.rs │
                            │.rs        │  │ (Filesystem)│ │ (Types) │
                            └───────────┘  └─────────────┘ └────┬────┘
                                                                 │
                            ┌────────────────────────────────────┘
                            │
                    (Utilisé partout)
```

## Modules

### `main.rs` - Point d'entrée
**Responsabilités:**
- Parsing des arguments CLI via `clap`
- Initialisation du logging
- Orchestration de haut niveau

**Code minimal et lisible** - délègue tout à `Packager`.

```rust
#[tokio::main]
async fn main() {
    init_logging();
    let args = Args::parse();
    let config = PackagerConfig::from_args(args);
    let packager = Packager::new(config);
    match packager.run().await { ... }
}
```

### `config.rs` - Configuration
**Types:**
- `Args`: Structure CLI parsée par clap
- `PackageInfo`: Info d'un package depuis package-lock.json
- `PackageLock`: Fichier package-lock.json complet
- `PackagerConfig`: Configuration normalisée et validée

**Responsabilités:**
- Parsing des arguments
- Validation de la configuration
- Centraliser tous les types de configuration

```rust
pub struct PackagerConfig {
    package_lock_path: PathBuf,
    output_dir: Option<PathBuf>,
    concurrent_downloads: usize,
    max_retries: u16,
}

impl PackagerConfig {
    pub fn validate(&self) -> PackagerResult<()> { ... }
}
```

### `error.rs` - Gestion d'erreur
**Type principal:**
- `PackagerError`: Enum avec variants domaine-spécifiques

**Avantages:**
- ✅ Gestion d'erreur uniforme
- ✅ Contexte enrichi (URL, chemin, raison)
- ✅ Type-safe (plus de `String` générique)
- ✅ Conversions automatiques depuis les erreurs stdlib

```rust
pub enum PackagerError {
    Download { url: String, reason: String },
    Integrity { url: String, reason: String },
    Io { path: PathBuf, reason: String },
    Config { reason: String },
    Parse { reason: String },
    Compression { path: PathBuf, reason: String },
}

// Type alias pour simplicité
pub type PackagerResult<T> = Result<T, PackagerError>;
```

### `packager.rs` - Orchestration métier
**Struct principale:**
- `Packager`: Orchestre tout le pipeline

**Responsabilités:**
- Lecture du package-lock.json
- Extraction des packages valides
- Gestion de la concurrence
- Rapport sur les échecs
- Compression du répertoire

```rust
pub struct Packager { config: PackagerConfig }

impl Packager {
    pub async fn run(&self) -> PackagerResult<String> {
        // 1. Valider config
        // 2. Lire lockfile
        // 3. Extraire packages
        // 4. Télécharger (concurrent)
        // 5. Rapport sur échecs
        // 6. Compression
        // 7. Nettoyage
    }
}
```

**Avantage:** Logique métier testable et réutilisable programmatiquement.

### `download.rs` - Téléchargement & Intégrité
**Types:**
- `Package`: Représente un package à télécharger

**Responsabilités:**
- Téléchargement avec retry et backoff exponentiel
- Vérification SHA512
- Gestion des erreurs réseau

```rust
pub async fn download_package_with_retry(
    package: &Package,
    output_dir: &Path,
    max_retry: u16,
) -> PackagerResult<()>
```

**Patterns:**
- Retry avec exponential backoff
- Erreurs spécifiques (Download vs Integrity)

### `system.rs` - Opérations filesystem
**Responsabilités:**
- Création de répertoires
- Génération de noms timestampés
- Compression en ZIP

```rust
pub fn zip_dir(dir_path: &Path, zip_path: &str) -> PackagerResult<()>
pub fn get_timestamped_dir() -> PathBuf
pub fn ensure_output_dir(dir_path: &Path) -> PackagerResult<()>
```

## Flux de données

```
CLI args
  ↓
Args::parse()
  ↓
PackagerConfig::from_args()
  ↓
Packager::new(config)
  ↓
Packager::run()
  ├─ config.validate()
  ├─ read_package_lock() → PackageLock
  ├─ extract_packages() → Vec<Package>
  ├─ download_packages() → Vec<failed_packages>
  │  └─ download_package_with_retry() × N (concurrent)
  ├─ report_failed_packages()
  ├─ zip_dir()
  └─ cleanup
```

## Avantages de cette architecture

| Aspect | Avant | Après |
|--------|-------|-------|
| **Gestion d'erreur** | `Box<dyn Error>` partout | `PackagerError` unifié |
| **Chemins** | `&str` pour les chemins | `PathBuf` / `Path` |
| **Logique métier** | Mélangée dans main.rs | Isolée dans `Packager` |
| **Testabilité** | Difficile | Facile (Packager struct) |
| **Réutilisabilité** | CLI only | API programmatique |
| **Validation** | Implicite | Explicite (`validate()`) |
| **Responsabilités** | Diffuses | Bien séparées (SRP) |
| **Logging** | eprintln/println | tracing |

## Testing

Pour tester les modules indépendamment:

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_validation() {
        let config = PackagerConfig { ... };
        assert!(config.validate().is_ok());
    }

    #[tokio::test]
    async fn test_packager() {
        let packager = Packager::new(config);
        let result = packager.run().await;
        assert!(result.is_ok());
    }
}
```

## Futur - Points d'extension

1. **Tests unitaires**: Ajouter `#[cfg(test)]` dans chaque module
2. **Retry strategy**: Rendre configurable via `config.rs`
3. **Multiple archive formats**: Ajouter variante zip/tar.gz dans `system.rs`
4. **Parallel downloads metrics**: Intégrer dans `packager.rs`
5. **Caching**: Ajouter module `cache.rs` avec logique de cache
6. **Database**: Pour tracker downloads/failures dans temps

## Commandes utiles

```bash
# Build
cargo build
cargo build --release

# Test
cargo test
cargo test --release

# Qualité
cargo clippy
cargo fmt

# Run
cargo run -- --help
cargo run -- --concurrent 200 --max-retries 5
```

