use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DocsgenError {
    #[error("I/O error for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse YAML file {path}: {source}")]
    Yaml {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
    #[error("invalid manifest {path}: {message}")]
    Manifest { path: PathBuf, message: String },
    #[error("missing or invalid markers in {path}")]
    MissingMarkers { path: PathBuf },
    #[error("Rust CLI contract does not define a `tq check` subcommand")]
    MissingCheckSubcommand,
}

impl DocsgenError {
    pub fn manifest(path: PathBuf, message: impl Into<String>) -> Self {
        Self::Manifest {
            path,
            message: message.into(),
        }
    }
}
