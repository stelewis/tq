use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("{message}")]
    Validation { message: String },
    #[error("failed to read config file {path}: {message}")]
    Read { path: PathBuf, message: String },
    #[error("invalid TOML in {path}: {message}")]
    Parse { path: PathBuf, message: String },
}

impl ConfigError {
    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }
}
