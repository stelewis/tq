use std::path::PathBuf;

use thiserror::Error;
use tq_core::RelativePathError;
use tq_discovery::DiscoveryError;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Rule engine received duplicate rule ids")]
    DuplicateRuleIds,
    #[error("Finding message must be non-empty")]
    EmptyFindingMessage,
    #[error("Finding line must be >= 1 when provided")]
    InvalidFindingLine,
    #[error("target test root must end with a directory name: {path}")]
    MissingTestRootDisplay { path: PathBuf },
    #[error("target test root display must be a non-empty relative path: {path}")]
    InvalidTestRootDisplay {
        path: PathBuf,
        #[source]
        source: RelativePathError,
    },
    #[error(transparent)]
    Discovery(#[from] DiscoveryError),
}
