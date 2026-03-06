# Guide du Logging dans npm_packager

## Vue d'ensemble

Le projet utilise **`tracing`** pour les logs structurés. C'est une fondation production-ready pour observabilité et debug.

## Utilisation Simple

### Mode Normal (pas de logs internes)
```bash
$ cargo run
Téléchargement de 137 packages en cours...
Compression des packages...
✓ Succès ! Archive créée: packages_20260306_213812.zip
```

### Debug Mode
```bash
$ RUST_LOG=debug cargo run
[DEBUG] Initializing packager configuration...
[DEBUG] Reading package-lock.json...
[DEBUG] Extracted 137 packages
[INFO] Downloading 137 packages...
[DEBUG] Downloading package-1.0.0...
[DEBUG] Downloading package-2.0.0...
...
[INFO] Compressing packages...
✓ Succès ! Archive créée: packages_20260306_213812.zip
```

## Niveaux de Log

### RUST_LOG=error
```bash
$ RUST_LOG=error cargo run
# Affiche seulement les erreurs
✗ Erreur: Le fichier package-lock.json n'existe pas: ...
```

### RUST_LOG=warn
```bash
$ RUST_LOG=warn cargo run
# Affiche warnings + erreurs
```

### RUST_LOG=info (défaut)
```bash
$ RUST_LOG=info cargo run
# Affiche infos + warnings + erreurs
```

### RUST_LOG=debug
```bash
$ RUST_LOG=debug cargo run
# Affiche tout avec détails techniques
```

### RUST_LOG=trace
```bash
$ RUST_LOG=trace cargo run
# Mode très verbeux (rarement utile)
```

## Filtrage par Module

### Un Seul Module
```bash
# Voir seulement les logs du download
$ RUST_LOG=npm_packager::download=debug cargo run

# Voir seulement les logs du packager
$ RUST_LOG=npm_packager::packager=debug cargo run
```

### Combinaison
```bash
# Download en debug, tout le reste en error
$ RUST_LOG=npm_packager::download=debug,error cargo run

# Packager en trace, download en info, autre en warn
$ RUST_LOG=npm_packager::packager=trace,npm_packager::download=info,warn cargo run
```

## Cas d'Usage Pratiques

### 🔧 Debug Pourquoi un Download Échoue
```bash
$ RUST_LOG=npm_packager::download=debug cargo run
# Affiche détails du retry, timeout, etc.
```

### 📊 CI/CD (Logs Minimalistes)
```bash
$ RUST_LOG=error cargo run
# Affiche seulement les erreurs critiques
```

### 🚀 Production (Info + Errors)
```bash
$ RUST_LOG=info cargo run
# Logs d'orchestration + erreurs
```

### 🐛 Audit Complet
```bash
$ RUST_LOG=debug cargo run 2>&1 | tee audit.log
# Sauvegarde tous les logs dans un fichier
```

## Architecture du Logging

### Initialisation (main.rs)
```rust
fn init_logging() {
    let env_filter = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::fmt()
        .with_env_filter(env_filter)
        .init();
}
```

**Comportement:**
- Lit `RUST_LOG` depuis l'environnement
- Défaut à `info` si non défini
- Format texte lisible par humains

### Où les Logs Proviennent

Actuellement, les logs structurés ne sont pas encore intégrés au code (c'est une tâche future).

Pour l'instant:
- ✅ Progress bars (`indicatif`) pour les téléchargements
- ✅ Spinner pour la compression
- ✅ Messages CLI avec `println!`/`eprintln!`
- 🔮 Logs tracing (infrastructure prête, en attente d'intégration)

## Points d'Extension Futurs

### 1. Ajouter des Logs dans download.rs
```rust
use tracing::{debug, warn, error};

pub async fn download_package_with_retry(...) {
    debug!(package = %package.name, "starting download");
    
    match download_package(...).await {
        Ok(_) => debug!(package = %package.name, "download successful"),
        Err(e) => warn!(package = %package.name, error = %e, "download failed, retrying"),
    }
}
```

### 2. Ajouter des Logs dans packager.rs
```rust
use tracing::info;

pub async fn run(&self) -> PackagerResult<String> {
    info!(total_packages = self.packages.len(), "starting batch download");
    // ...
    info!(successful = ok_count, failed = failed_count, "batch completed");
}
```

### 3. JSON Logs (Production)
```rust
// Dans init_logging()
tracing_subscriber::fmt()
    .json()  // ← Ajouter cette ligne
    .with_env_filter(env_filter)
    .init();
```

Puis l'utiliser:
```bash
$ RUST_LOG=info cargo run | jq .
# Les logs deviennent parsables en JSON
```

### 4. Sentry Integration (Error Tracking)
```rust
let _guard = sentry::init("https://...");

// Les erreurs se retrouvent automatiquement dans Sentry
```

## Recommandations

### Pour les Développeurs
- Utiliser `RUST_LOG=debug` pendant le développement
- Tester avec `RUST_LOG=error` pour simuler la production

### Pour les Utilisateurs
- Mode normal: `cargo run` (pas de logs internes)
- Mode debug: `RUST_LOG=debug cargo run` si problème

### Pour la Production
```bash
# Logs seulement errors + output vers fichier
RUST_LOG=error ./npm_packager >> packages.log 2>&1

# Ou avec timestamps
RUST_LOG=info ./npm_packager | ts '[%Y-%m-%d %H:%M:%S]' >> packages.log
```

## Résumé

| Cas | Commande |
|-----|----------|
| Normal | `cargo run` |
| Debug | `RUST_LOG=debug cargo run` |
| Production | `RUST_LOG=info ./npm_packager` |
| CI/CD | `RUST_LOG=error ./npm_packager` |
| Filtrer module | `RUST_LOG=npm_packager::download=debug cargo run` |
| JSON logs | `RUST_LOG=info cargo run 2>&1 \| jq` |

Tracing prépare le projet pour escalabilité et observabilité future ! 📊
