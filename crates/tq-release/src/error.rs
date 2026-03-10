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
    #[error("failed to parse yaml file {path}: {source}")]
    Yaml {
        path: PathBuf,
        #[source]
        source: serde_yaml::Error,
    },
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
