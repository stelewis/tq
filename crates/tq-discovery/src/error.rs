use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("{path} must be an existing directory")]
    NotDirectory { path: PathBuf },
    #[error("{operation} failed for {path}: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{message}")]
    Validation { message: String },
}
