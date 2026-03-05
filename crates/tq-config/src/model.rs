use std::path::PathBuf;

use crate::ConfigError;

pub const DEFAULT_IGNORE_INIT_MODULES: bool = false;
pub const DEFAULT_MAX_TEST_FILE_NON_BLANK_LINES: u64 = 600;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum QualifierStrategy {
    None,
    AnySuffix,
    Allowlist,
}

impl QualifierStrategy {
    pub(crate) const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::AnySuffix => "any-suffix",
            Self::Allowlist => "allowlist",
        }
    }

    pub(crate) fn parse(raw: &str) -> Option<Self> {
        match raw {
            "none" => Some(Self::None),
            "any-suffix" => Some(Self::AnySuffix),
            "allowlist" => Some(Self::Allowlist),
            _ => None,
        }
    }
}

impl Default for QualifierStrategy {
    fn default() -> Self {
        Self::AnySuffix
    }
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RuleId(String);

impl RuleId {
    pub fn parse(value: &str) -> Result<Self, ConfigError> {
        if value.is_empty() {
            return Err(ConfigError::validation("RuleId must be non-empty"));
        }

        let mut chars = value.chars();
        let Some(first) = chars.next() else {
            return Err(ConfigError::validation("RuleId must be non-empty"));
        };

        if !first.is_ascii_lowercase() {
            return Err(ConfigError::validation(
                "RuleId must be kebab-case, e.g. mapping-missing-test",
            ));
        }

        let mut previous_was_dash = false;
        for character in chars {
            if character == '-' {
                if previous_was_dash {
                    return Err(ConfigError::validation(
                        "RuleId must be kebab-case, e.g. mapping-missing-test",
                    ));
                }
                previous_was_dash = true;
                continue;
            }

            if !character.is_ascii_lowercase() && !character.is_ascii_digit() {
                return Err(ConfigError::validation(
                    "RuleId must be kebab-case, e.g. mapping-missing-test",
                ));
            }

            previous_was_dash = false;
        }

        if previous_was_dash {
            return Err(ConfigError::validation(
                "RuleId must be kebab-case, e.g. mapping-missing-test",
            ));
        }

        Ok(Self(value.to_owned()))
    }
}

impl std::fmt::Display for RuleId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0)
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct PartialRuleConfig {
    pub ignore_init_modules: Option<bool>,
    pub max_test_file_non_blank_lines: Option<u64>,
    pub qualifier_strategy: Option<QualifierStrategy>,
    pub allowed_qualifiers: Option<Vec<String>>,
    pub select: Option<Vec<RuleId>>,
    pub ignore: Option<Vec<RuleId>>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct PartialTargetConfig {
    pub name: Option<String>,
    pub package: Option<String>,
    pub source_root: Option<String>,
    pub test_root: Option<String>,
    pub ignore_init_modules: Option<bool>,
    pub max_test_file_non_blank_lines: Option<u64>,
    pub qualifier_strategy: Option<QualifierStrategy>,
    pub allowed_qualifiers: Option<Vec<String>>,
    pub select: Option<Vec<RuleId>>,
    pub ignore: Option<Vec<RuleId>>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct PartialTqConfig {
    pub defaults: PartialRuleConfig,
    pub targets: Option<Vec<PartialTargetConfig>>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct CliOverrides {
    pub ignore_init_modules: Option<bool>,
    pub max_test_file_non_blank_lines: Option<u64>,
    pub qualifier_strategy: Option<QualifierStrategy>,
    pub allowed_qualifiers: Option<Vec<String>>,
    pub select: Option<Vec<RuleId>>,
    pub ignore: Option<Vec<RuleId>>,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TqTargetConfig {
    pub name: String,
    pub package: String,
    pub source_root: PathBuf,
    pub test_root: PathBuf,
    pub ignore_init_modules: bool,
    pub max_test_file_non_blank_lines: u64,
    pub qualifier_strategy: QualifierStrategy,
    pub allowed_qualifiers: Vec<String>,
    pub select: Vec<RuleId>,
    pub ignore: Vec<RuleId>,
}

impl TqTargetConfig {
    #[must_use]
    pub fn package_path(&self) -> PathBuf {
        self.package
            .split('.')
            .fold(PathBuf::new(), |path, segment| path.join(segment))
    }

    #[must_use]
    pub fn source_package_root(&self) -> PathBuf {
        normalize_absolute(&self.source_root.join(self.package_path()))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TqConfig {
    pub targets: Vec<TqTargetConfig>,
}

fn normalize_absolute(path: &std::path::Path) -> PathBuf {
    use std::path::Component;

    let mut normalized = PathBuf::new();
    for component in path.components() {
        match component {
            Component::ParentDir => {
                let _ = normalized.pop();
            }
            Component::CurDir => {}
            other => normalized.push(other.as_os_str()),
        }
    }
    normalized
}
