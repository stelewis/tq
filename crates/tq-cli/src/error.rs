use std::path::Path;

use thiserror::Error;
use tq_config::ConfigError;
use tq_core::RuleIdError;
use tq_discovery::DiscoveryError;
use tq_engine::EngineError;
use tq_reporting::ReportingError;
use tq_rules::RulesError;

#[derive(Debug, Error)]
pub enum CliError {
    #[error("Invalid rule ID {value:?}: {source}")]
    InvalidRuleId {
        value: String,
        #[source]
        source: RuleIdError,
    },
    #[error("Duplicate rule ID in CLI values: {rule_id}")]
    DuplicateCliRuleId { rule_id: String },
    #[error("Invalid --severity value '{value}': expected RULE_ID=SEVERITY")]
    MalformedSeverityOverride { value: String },
    #[error(
        "Invalid severity '{severity}' in --severity {value}: expected error, warning, or info"
    )]
    UnknownSeverity { severity: String, value: String },
    #[error("Unknown target name(s): {names}")]
    UnknownTargetNames { names: String },
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
    Discovery(#[from] DiscoveryError),
    #[error(transparent)]
    Engine(#[from] EngineError),
    #[error(transparent)]
    Rules(#[from] RulesError),
    #[error(transparent)]
    Reporting(#[from] ReportingError),
}

impl CliError {
    pub fn invalid_rule_id(value: &str, source: RuleIdError) -> Self {
        Self::InvalidRuleId {
            value: value.to_owned(),
            source,
        }
    }

    pub fn duplicate_cli_rule_id(rule_id: impl Into<String>) -> Self {
        Self::DuplicateCliRuleId {
            rule_id: rule_id.into(),
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
