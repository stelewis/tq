use std::path::Path;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("configuration path does not exist: {path}")]
    MissingConfigPath { path: String },
    #[error("provided --config path is not a file: {path}")]
    ConfigPathNotFile { path: String },
    #[error("--isolated cannot be combined with --config (provided: {path})")]
    IsolatedWithConfig { path: String },
}

impl CliError {
    pub fn from_missing_config(path: &Path) -> Self {
        Self::MissingConfigPath {
            path: path.display().to_string(),
        }
    }

    pub fn from_non_file_config(path: &Path) -> Self {
        Self::ConfigPathNotFile {
            path: path.display().to_string(),
        }
    }

    pub fn from_isolated_with_config(path: &Path) -> Self {
        Self::IsolatedWithConfig {
            path: path.display().to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, CliError>;
