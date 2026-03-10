use std::path::Path;

use thiserror::Error;
use tq_config::ConfigError;
use tq_engine::EngineError;
use tq_reporting::ReportingError;
use tq_rules::RulesError;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("{message}")]
    Validation { message: String },
    #[error("configuration path does not exist: {path}")]
    MissingConfigPath { path: String },
    #[error("provided --config path is not a file: {path}")]
    ConfigPathNotFile { path: String },
    #[error("--isolated cannot be combined with --config (provided: {path})")]
    IsolatedWithConfig { path: String },
    #[error("failed to resolve current directory: {message}")]
    CurrentDirectory { message: String },
    #[error("Configured source package root does not exist for target '{target}': {path}")]
    MissingSourcePackageRoot { target: String, path: String },
    #[error("Configured test root does not exist for target '{target}': {path}")]
    MissingTestRoot { target: String, path: String },
    #[error(transparent)]
    Config(#[from] ConfigError),
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Rules(#[from] RulesError),
    #[error(transparent)]
    Reporting(#[from] ReportingError),
}

impl CliError {
    pub fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }

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

    pub fn from_current_dir(error: &std::io::Error) -> Self {
        Self::CurrentDirectory {
            message: error.to_string(),
        }
    }

    pub fn from_missing_source_package_root(target: &str, path: &Path) -> Self {
        Self::MissingSourcePackageRoot {
            target: target.to_owned(),
            path: path.display().to_string(),
        }
    }

    pub fn from_missing_test_root(target: &str, path: &Path) -> Self {
        Self::MissingTestRoot {
            target: target.to_owned(),
            path: path.display().to_string(),
        }
    }
}

pub type Result<T> = std::result::Result<T, CliError>;
