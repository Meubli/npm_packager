use std::fmt;
use std::io;
use std::path::PathBuf;

/// Erreurs spécifiques au packager
#[derive(Debug)]
pub enum PackagerError {
    /// Erreur lors du téléchargement d'un package
    Download { url: String, reason: String },
    /// Erreur lors de la vérification d'intégrité
    Integrity { url: String, reason: String },
    /// Erreur d'entrée/sortie filesystem
    Io { path: PathBuf, reason: String },
    /// Erreur de configuration
    Config { reason: String },
    /// Erreur de parsing du fichier package-lock.json
    Parse { reason: String },
    /// Erreur lors de la compression
    Compression { path: PathBuf, reason: String },
}

impl fmt::Display for PackagerError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            PackagerError::Download { url, reason } => {
                write!(f, "Erreur de téléchargement [{}]: {}", url, reason)
            }
            PackagerError::Integrity { url, reason } => {
                write!(f, "Erreur d'intégrité [{}]: {}", url, reason)
            }
            PackagerError::Io { path, reason } => {
                write!(f, "Erreur filesystem [{}]: {}", path.display(), reason)
            }
            PackagerError::Config { reason } => {
                write!(f, "Erreur de configuration: {}", reason)
            }
            PackagerError::Parse { reason } => {
                write!(f, "Erreur de parsing JSON: {}", reason)
            }
            PackagerError::Compression { path, reason } => {
                write!(f, "Erreur de compression [{}]: {}", path.display(), reason)
            }
        }
    }
}

impl std::error::Error for PackagerError {}

// Conversions pour faciliter la conversion d'erreurs
impl From<io::Error> for PackagerError {
    fn from(err: io::Error) -> Self {
        PackagerError::Io {
            path: PathBuf::from("<unknown>"),
            reason: err.to_string(),
        }
    }
}

impl From<serde_json::Error> for PackagerError {
    fn from(err: serde_json::Error) -> Self {
        PackagerError::Parse {
            reason: err.to_string(),
        }
    }
}

impl From<String> for PackagerError {
    fn from(reason: String) -> Self {
        PackagerError::Config { reason }
    }
}

impl From<&str> for PackagerError {
    fn from(reason: &str) -> Self {
        PackagerError::Config {
            reason: reason.to_string(),
        }
    }
}

/// Type alias pour simplifier les signatures de fonctions
pub type PackagerResult<T> = Result<T, PackagerError>;
