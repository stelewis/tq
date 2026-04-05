use std::path::PathBuf;

use thiserror::Error;
use tq_core::{PackageNameError, RuleIdError, TargetNameError};

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("Missing required configuration key: tool.tq.targets")]
    MissingTargets,
    #[error("Config file not found: {path}")]
    ConfigNotFound { path: PathBuf },
    #[error("TOML document must be a table")]
    DocumentMustBeTable,
    #[error("Missing [tool.tq] section in config file: {path}")]
    MissingToolTqSection { path: PathBuf },
    #[error("{location} must be a table")]
    TableExpected { location: String },
    #[error("{prefix} {keys}")]
    UnknownKeys { prefix: String, keys: String },
    #[error("{location} must be a string")]
    StringExpected { location: String },
    #[error("{location} must be non-empty")]
    EmptyString { location: String },
    #[error("{location} must be one of: {expected}")]
    InvalidEnumValue { location: String, expected: String },
    #[error("{location} must be an integer")]
    IntegerExpected { location: String },
    #[error("{location} must be >= 1")]
    PositiveIntegerExpected { location: String },
    #[error("{location} must be an array of strings")]
    StringArrayExpected { location: String },
    #[error("{location} must be an array of tables")]
    TableArrayExpected { location: String },
    #[error(
        "{location} contains duplicate value {value:?} at indices {first_index} and {second_index}"
    )]
    DuplicateValue {
        location: String,
        value: String,
        first_index: usize,
        second_index: usize,
    },
    #[error("{location} contains invalid rule id: {value}")]
    InvalidRuleId {
        location: String,
        value: String,
        #[source]
        source: RuleIdError,
    },
    #[error("Missing required target key: {location}")]
    MissingTargetKey { location: String },
    #[error("{location} must be kebab-case: {value}")]
    InvalidTargetName {
        location: String,
        value: String,
        #[source]
        source: TargetNameError,
    },
    #[error("{location} must be dotted Python identifiers")]
    InvalidPackageName {
        location: String,
        value: String,
        #[source]
        source: PackageNameError,
    },
    #[error("{location} must not contain platform path prefixes: {value}")]
    InvalidTargetPathPrefix { location: String, value: String },
    #[error("{location} must be non-empty when effective qualifier_strategy is 'allowlist'")]
    AllowlistRequiresQualifiers { location: String },
    #[error(
        "Duplicate target name in tool.tq.targets[{first_index}].name and tool.tq.targets[{second_index}].name: {name}"
    )]
    DuplicateTargetName {
        first_index: usize,
        second_index: usize,
        name: String,
    },
    #[error(
        "Duplicate source package root across tool.tq.targets[{first_index}] and tool.tq.targets[{second_index}]: {path}"
    )]
    DuplicateSourcePackageRoot {
        first_index: usize,
        second_index: usize,
        path: PathBuf,
    },
    #[error("failed to resolve current directory: {message}")]
    CurrentDirectory { message: String },
    #[error("failed to read config file {path}: {message}")]
    Read { path: PathBuf, message: String },
    #[error("invalid TOML in {path}: {message}")]
    Parse { path: PathBuf, message: String },
}
