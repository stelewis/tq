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
    #[error("index file paths must not contain platform path prefixes: {path}")]
    PrefixedIndexPath { path: PathBuf },
    #[error("index file paths must be relative: {path}")]
    AbsoluteIndexPath { path: PathBuf },
    #[error("index file paths must not contain '.': {path}")]
    CurrentDirIndexPath { path: PathBuf },
    #[error("index file paths must not contain '..': {path}")]
    ParentDirIndexPath { path: PathBuf },
    #[error("discovered file {path} is not under root {root}")]
    DiscoveredPathOutsideRoot { path: PathBuf, root: PathBuf },
}
