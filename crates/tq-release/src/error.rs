use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReleaseError {
    #[error("distribution directory does not exist: {path}")]
    MissingDistributionDirectory { path: PathBuf },
    #[error("I/O error for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid Dependabot config {path}: {message}")]
    DependabotConfig { path: PathBuf, message: String },
    #[error("failed to read zip archive {path}: {source}")]
    Zip {
        path: PathBuf,
        #[source]
        source: zip::result::ZipError,
    },
    #[error("artifact content policy check failed. Forbidden paths were found:\n{details}")]
    PolicyViolation { details: String },
    #[error("repository policy check failed:\n{details}")]
    RepositoryPolicyViolation { details: String },
}
