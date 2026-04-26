use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum ReleaseError {
    #[error("distribution directory does not exist: {path}")]
    MissingDistributionDirectory { path: PathBuf },
    #[error("failed to run git command in {repo_root}: git {args}: {source}")]
    GitIo {
        repo_root: PathBuf,
        args: String,
        #[source]
        source: std::io::Error,
    },
    #[error("git command failed in {repo_root}: git {args}\n{stderr}")]
    Git {
        repo_root: PathBuf,
        args: String,
        stderr: String,
    },
    #[error("I/O error for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid Dependabot config {path}: {message}")]
    DependabotConfig { path: PathBuf, message: String },
    #[error("invalid release intent input {path}: {message}")]
    ReleaseIntentInput { path: PathBuf, message: String },
    #[error("invalid workspace version input {path}: {message}")]
    WorkspaceVersionInput { path: PathBuf, message: String },
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
