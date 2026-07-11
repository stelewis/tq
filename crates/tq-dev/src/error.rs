use std::path::PathBuf;

use thiserror::Error;

#[derive(Debug, Error)]
pub enum DevError {
    #[error("distribution directory does not exist: {path}")]
    MissingDistributionDirectory { path: PathBuf },
    #[error("failed to run git command: git {args}: {source}")]
    GitIo {
        args: String,
        #[source]
        source: std::io::Error,
    },
    #[error("git command failed: git {args}\n{stderr}")]
    Git { args: String, stderr: String },
    #[error("failed to run command: {program} {args:?}: {source}")]
    CommandIo {
        program: String,
        args: Vec<String>,
        #[source]
        source: std::io::Error,
    },
    #[error("command failed: {program} {args:?} exited with {code:?}")]
    CommandFailed {
        program: String,
        args: Vec<String>,
        code: Option<i32>,
    },
    #[error("could not parse output of {program}: {message}")]
    CommandOutputParse { program: String, message: String },
    #[error("I/O error for {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("invalid input {path}: {message}")]
    InvalidInput { path: PathBuf, message: String },
    #[error("failed to serialize report to JSON: {source}")]
    ReportSerialization {
        #[source]
        source: serde_json::Error,
    },
    #[error("failed to read zip archive {path}: {source}")]
    Zip {
        path: PathBuf,
        #[source]
        source: zip::result::ZipError,
    },
    #[error("artifact content policy check failed. Forbidden paths were found:\n{details}")]
    ArtifactPolicyViolation { details: String },
    #[error("repository policy check failed:\n{details}")]
    PolicyViolation { details: String },
    #[error("missing native build prerequisites:\n{details}")]
    MissingBuildPrerequisites { details: String },
}
