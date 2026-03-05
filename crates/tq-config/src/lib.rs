use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;
use toml::Table;

pub const DEFAULT_IGNORE_INIT_MODULES: bool = false;
pub const DEFAULT_MAX_TEST_FILE_NON_BLANK_LINES: u64 = 600;

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum QualifierStrategy {
    None,
    AnySuffix,
    Allowlist,
}

impl QualifierStrategy {
    const fn as_str(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::AnySuffix => "any-suffix",
            Self::Allowlist => "allowlist",
        }
    }

    fn parse(raw: &str) -> Option<Self> {
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

#[derive(Debug, Error)]
pub enum ConfigError {
    #[error("{message}")]
    Validation { message: String },
    #[error("failed to read config file {path}: {message}")]
    Read { path: PathBuf, message: String },
    #[error("invalid TOML in {path}: {message}")]
    Parse { path: PathBuf, message: String },
}

impl ConfigError {
    fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
    }
}

pub fn resolve_tq_config(
    cwd: &Path,
    explicit_config_path: Option<&Path>,
    isolated: bool,
    cli_overrides: &CliOverrides,
) -> Result<TqConfig, ConfigError> {
    let cwd = absolute_from_process(cwd)?;
    let mut discovered = PartialTqConfig::default();
    let mut targets_base_dir: Option<PathBuf> = None;

    if let Some(explicit_path) = explicit_config_path {
        let config_path = absolute_from_process(explicit_path)?;
        let loaded = load_partial_from_pyproject(&config_path, true)?;
        targets_base_dir = resolve_targets_base_dir(targets_base_dir, &loaded, &config_path);
        discovered = loaded;
    } else if !isolated {
        let user_config_path =
            home_dir().map(|home| home.join(".config").join("tq").join("pyproject.toml"));
        let project_config_path = find_project_pyproject(&cwd);

        if let Some(user_config_path) = user_config_path.filter(|path| path.exists()) {
            let user_partial = load_partial_from_pyproject(&user_config_path, false)?;
            targets_base_dir =
                resolve_targets_base_dir(targets_base_dir, &user_partial, &user_config_path);
            discovered = merge_partial(&discovered, &user_partial);
        }

        if let Some(project_config_path) = project_config_path {
            let project_partial = load_partial_from_pyproject(&project_config_path, false)?;
            targets_base_dir =
                resolve_targets_base_dir(targets_base_dir, &project_partial, &project_config_path);
            discovered = merge_partial(&discovered, &project_partial);
        }
    }

    let cli_partial = partial_from_cli(cli_overrides)?;
    materialize_config(
        &cwd,
        &discovered,
        &cli_partial.defaults,
        targets_base_dir.as_deref(),
    )
}

fn home_dir() -> Option<PathBuf> {
    std::env::var_os("HOME").map(PathBuf::from)
}

fn absolute_from_process(path: &Path) -> Result<PathBuf, ConfigError> {
    if path.is_absolute() {
        return Ok(normalize_absolute(path));
    }

    let current = std::env::current_dir().map_err(|error| {
        ConfigError::validation(format!("failed to resolve current directory: {error}"))
    })?;
    Ok(normalize_absolute(&current.join(path)))
}

fn resolve_targets_base_dir(
    current: Option<PathBuf>,
    loaded: &PartialTqConfig,
    config_path: &Path,
) -> Option<PathBuf> {
    if loaded.targets.is_none() {
        return current;
    }
    config_path.parent().map(normalize_absolute)
}

fn find_project_pyproject(cwd: &Path) -> Option<PathBuf> {
    for candidate in cwd.ancestors() {
        let pyproject = candidate.join("pyproject.toml");
        if pyproject.exists() {
            return Some(pyproject);
        }
    }
    None
}

fn load_partial_from_pyproject(
    path: &Path,
    require_section: bool,
) -> Result<PartialTqConfig, ConfigError> {
    if !path.exists() {
        return Err(ConfigError::validation(format!(
            "Config file not found: {}",
            path.display()
        )));
    }

    let raw = fs::read_to_string(path).map_err(|error| ConfigError::Read {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;
    let document = toml::from_str::<toml::Value>(&raw).map_err(|error| ConfigError::Parse {
        path: path.to_path_buf(),
        message: error.to_string(),
    })?;

    let Some(root_table) = document.as_table() else {
        return Err(ConfigError::validation("TOML document must be a table"));
    };

    let Some(tool_value) = root_table.get("tool") else {
        if require_section {
            return Err(ConfigError::validation(format!(
                "Missing [tool.tq] section in config file: {}",
                path.display()
            )));
        }
        return Ok(PartialTqConfig::default());
    };

    let tool_table = as_table(tool_value, "[tool]")?;
    let Some(tq_value) = tool_table.get("tq") else {
        if require_section {
            return Err(ConfigError::validation(format!(
                "Missing [tool.tq] section in config file: {}",
                path.display()
            )));
        }
        return Ok(PartialTqConfig::default());
    };

    let tq_table = as_table(tq_value, "[tool.tq]")?;
    reject_unknown_keys(
        tq_table,
        &[
            "ignore_init_modules",
            "max_test_file_non_blank_lines",
            "qualifier_strategy",
            "allowed_qualifiers",
            "select",
            "ignore",
            "targets",
        ],
        "Unknown [tool.tq] key(s):",
    )?;

    let defaults = PartialRuleConfig {
        ignore_init_modules: expect_optional_bool(tq_table, "ignore_init_modules", "tool.tq")?,
        max_test_file_non_blank_lines: expect_optional_positive_int(
            tq_table,
            "max_test_file_non_blank_lines",
            "tool.tq",
        )?,
        qualifier_strategy: expect_optional_qualifier_strategy(
            tq_table,
            "qualifier_strategy",
            "tool.tq",
        )?,
        allowed_qualifiers: expect_optional_string_list(
            tq_table,
            "allowed_qualifiers",
            "tool.tq",
            true,
        )?,
        select: expect_optional_rule_ids(tq_table, "select", "tool.tq")?,
        ignore: expect_optional_rule_ids(tq_table, "ignore", "tool.tq")?,
    };

    let targets = expect_optional_targets(tq_table, "targets")?;
    Ok(PartialTqConfig { defaults, targets })
}

fn partial_from_cli(overrides: &CliOverrides) -> Result<PartialTqConfig, ConfigError> {
    if let Some(values) = &overrides.allowed_qualifiers {
        raise_on_duplicate_strings(values, "cli.allowed_qualifiers")?;
    }

    Ok(PartialTqConfig {
        defaults: PartialRuleConfig {
            ignore_init_modules: overrides.ignore_init_modules,
            max_test_file_non_blank_lines: overrides.max_test_file_non_blank_lines,
            qualifier_strategy: overrides.qualifier_strategy,
            allowed_qualifiers: overrides.allowed_qualifiers.clone(),
            select: overrides.select.clone(),
            ignore: overrides.ignore.clone(),
        },
        targets: None,
    })
}

fn merge_partial(base: &PartialTqConfig, override_: &PartialTqConfig) -> PartialTqConfig {
    PartialTqConfig {
        defaults: merge_rule_partial(&base.defaults, &override_.defaults),
        targets: override_.targets.clone().or_else(|| base.targets.clone()),
    }
}

fn merge_rule_partial(
    base: &PartialRuleConfig,
    override_: &PartialRuleConfig,
) -> PartialRuleConfig {
    PartialRuleConfig {
        ignore_init_modules: override_.ignore_init_modules.or(base.ignore_init_modules),
        max_test_file_non_blank_lines: override_
            .max_test_file_non_blank_lines
            .or(base.max_test_file_non_blank_lines),
        qualifier_strategy: override_.qualifier_strategy.or(base.qualifier_strategy),
        allowed_qualifiers: override_
            .allowed_qualifiers
            .clone()
            .or_else(|| base.allowed_qualifiers.clone()),
        select: override_.select.clone().or_else(|| base.select.clone()),
        ignore: override_.ignore.clone().or_else(|| base.ignore.clone()),
    }
}

fn materialize_config(
    cwd: &Path,
    partial: &PartialTqConfig,
    cli_defaults: &PartialRuleConfig,
    targets_base_dir: Option<&Path>,
) -> Result<TqConfig, ConfigError> {
    let Some(targets) = &partial.targets else {
        return Err(ConfigError::validation(
            "Missing required configuration key: tool.tq.targets",
        ));
    };
    if targets.is_empty() {
        return Err(ConfigError::validation(
            "Missing required configuration key: tool.tq.targets",
        ));
    }

    let base_dir = targets_base_dir.map_or_else(|| cwd.to_path_buf(), PathBuf::from);

    let mut normalized_targets = Vec::with_capacity(targets.len());
    let mut seen_names: BTreeMap<String, usize> = BTreeMap::new();
    let mut seen_roots: BTreeMap<PathBuf, usize> = BTreeMap::new();

    for (target_index, target) in targets.iter().enumerate() {
        let resolved = materialize_target(
            &base_dir,
            target,
            &partial.defaults,
            cli_defaults,
            target_index,
        )?;

        if let Some(first_index) = seen_names.get(&resolved.name) {
            return Err(ConfigError::validation(format!(
                "Duplicate target name in tool.tq.targets[{first_index}].name and tool.tq.targets[{target_index}].name: {}",
                resolved.name
            )));
        }
        seen_names.insert(resolved.name.clone(), target_index);

        let source_package_root = resolved.source_package_root();
        if let Some(first_index) = seen_roots.get(&source_package_root) {
            return Err(ConfigError::validation(format!(
                "Duplicate source package root across tool.tq.targets[{first_index}] and tool.tq.targets[{target_index}]: {}",
                source_package_root.display()
            )));
        }
        seen_roots.insert(source_package_root, target_index);

        normalized_targets.push(resolved);
    }

    normalized_targets.sort_by(|left, right| left.name.cmp(&right.name));
    Ok(TqConfig {
        targets: normalized_targets,
    })
}

fn materialize_target(
    targets_base_dir: &Path,
    target: &PartialTargetConfig,
    defaults: &PartialRuleConfig,
    cli_defaults: &PartialRuleConfig,
    target_index: usize,
) -> Result<TqTargetConfig, ConfigError> {
    let location = format!("tool.tq.targets[{target_index}]");

    let name = require_target_key(target.name.as_ref(), "name", &location)?;
    if !is_kebab_case_target_name(&name) {
        return Err(ConfigError::validation(format!(
            "{location}.name must be kebab-case: {name}"
        )));
    }

    let package = require_target_key(target.package.as_ref(), "package", &location)?;
    validate_python_package_name(&package, &format!("{location}.package"))?;

    let source_root_value =
        require_target_key(target.source_root.as_ref(), "source_root", &location)?;
    let test_root_value = require_target_key(target.test_root.as_ref(), "test_root", &location)?;

    let merged_rules = merge_rule_partial(
        defaults,
        &PartialRuleConfig {
            ignore_init_modules: target.ignore_init_modules,
            max_test_file_non_blank_lines: target.max_test_file_non_blank_lines,
            qualifier_strategy: target.qualifier_strategy,
            allowed_qualifiers: target.allowed_qualifiers.clone(),
            select: target.select.clone(),
            ignore: target.ignore.clone(),
        },
    );
    let final_rules = merge_rule_partial(&merged_rules, cli_defaults);

    let allowed_qualifiers = final_rules.allowed_qualifiers.unwrap_or_default();
    let qualifier_strategy = final_rules.qualifier_strategy.unwrap_or_default();

    if qualifier_strategy == QualifierStrategy::Allowlist && allowed_qualifiers.is_empty() {
        return Err(ConfigError::validation(format!(
            "{location}.allowed_qualifiers must be non-empty when effective qualifier_strategy is 'allowlist'"
        )));
    }

    Ok(TqTargetConfig {
        name,
        package,
        source_root: resolve_path(targets_base_dir, &source_root_value),
        test_root: resolve_path(targets_base_dir, &test_root_value),
        ignore_init_modules: final_rules
            .ignore_init_modules
            .unwrap_or(DEFAULT_IGNORE_INIT_MODULES),
        max_test_file_non_blank_lines: final_rules
            .max_test_file_non_blank_lines
            .unwrap_or(DEFAULT_MAX_TEST_FILE_NON_BLANK_LINES),
        qualifier_strategy,
        allowed_qualifiers,
        select: final_rules.select.unwrap_or_default(),
        ignore: final_rules.ignore.unwrap_or_default(),
    })
}

fn require_target_key(
    value: Option<&String>,
    key: &str,
    location: &str,
) -> Result<String, ConfigError> {
    let Some(value) = value else {
        return Err(ConfigError::validation(format!(
            "Missing required target key: {location}.{key}"
        )));
    };
    if value.trim().is_empty() {
        return Err(ConfigError::validation(format!(
            "{location}.{key} must be non-empty"
        )));
    }
    Ok(value.clone())
}

fn validate_python_package_name(value: &str, location: &str) -> Result<(), ConfigError> {
    let segments: Vec<&str> = value.split('.').collect();
    if segments
        .iter()
        .any(|segment| !is_python_identifier(segment))
    {
        return Err(ConfigError::validation(format!(
            "{location} must be dotted Python identifiers"
        )));
    }
    Ok(())
}

fn is_python_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };

    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }

    characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

fn is_kebab_case_target_name(value: &str) -> bool {
    if value.is_empty() {
        return false;
    }

    let mut has_segment_char = false;
    let mut previous_was_dash = false;
    for character in value.chars() {
        if character == '-' {
            if !has_segment_char {
                return false;
            }
            if previous_was_dash {
                return false;
            }
            previous_was_dash = true;
            continue;
        }

        if !character.is_ascii_lowercase() && !character.is_ascii_digit() {
            return false;
        }
        has_segment_char = true;
        previous_was_dash = false;
    }

    !previous_was_dash
}

fn resolve_path(base_dir: &Path, value: &str) -> PathBuf {
    let candidate = Path::new(value);
    if candidate.is_absolute() {
        return normalize_absolute(candidate);
    }
    normalize_absolute(&base_dir.join(candidate))
}

fn normalize_absolute(path: &Path) -> PathBuf {
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

fn as_table<'a>(value: &'a toml::Value, location: &str) -> Result<&'a Table, ConfigError> {
    value.as_table().ok_or_else(|| {
        let message = if location == "[tool.tq]" {
            "[tool.tq] must be a table".to_owned()
        } else {
            format!("{location} must be a table")
        };
        ConfigError::validation(message)
    })
}

fn reject_unknown_keys(table: &Table, expected: &[&str], prefix: &str) -> Result<(), ConfigError> {
    let expected: BTreeSet<&str> = expected.iter().copied().collect();
    let unknown: Vec<&str> = table
        .keys()
        .map(String::as_str)
        .filter(|key| !expected.contains(key))
        .collect();

    if unknown.is_empty() {
        return Ok(());
    }

    let keys = unknown.join(", ");
    Err(ConfigError::validation(format!("{prefix} {keys}")))
}

fn expect_optional_str(
    table: &Table,
    key: &str,
    location: &str,
) -> Result<Option<String>, ConfigError> {
    let Some(value) = table.get(key) else {
        return Ok(None);
    };

    let Some(string) = value.as_str() else {
        return Err(ConfigError::validation(format!(
            "{location}.{key} must be a string"
        )));
    };
    if string.trim().is_empty() {
        return Err(ConfigError::validation(format!(
            "{location}.{key} must be non-empty"
        )));
    }
    Ok(Some(string.to_owned()))
}

fn expect_optional_bool(
    table: &Table,
    key: &str,
    location: &str,
) -> Result<Option<bool>, ConfigError> {
    let Some(value) = table.get(key) else {
        return Ok(None);
    };
    value
        .as_bool()
        .map(Some)
        .ok_or_else(|| ConfigError::validation(format!("{location}.{key} must be a boolean")))
}

fn expect_optional_positive_int(
    table: &Table,
    key: &str,
    location: &str,
) -> Result<Option<u64>, ConfigError> {
    let Some(value) = table.get(key) else {
        return Ok(None);
    };

    let Some(integer) = value.as_integer() else {
        return Err(ConfigError::validation(format!(
            "{location}.{key} must be an integer"
        )));
    };
    if integer < 1 {
        return Err(ConfigError::validation(format!(
            "{location}.{key} must be >= 1"
        )));
    }
    let integer = u64::try_from(integer)
        .map_err(|_| ConfigError::validation(format!("{location}.{key} must be >= 1")))?;
    Ok(Some(integer))
}

fn expect_optional_qualifier_strategy(
    table: &Table,
    key: &str,
    location: &str,
) -> Result<Option<QualifierStrategy>, ConfigError> {
    let Some(value) = table.get(key) else {
        return Ok(None);
    };

    let Some(raw) = value.as_str() else {
        return Err(ConfigError::validation(format!(
            "{location}.{key} must be a string"
        )));
    };

    QualifierStrategy::parse(raw).map(Some).ok_or_else(|| {
        ConfigError::validation(format!(
            "{location}.{key} must be one of: {}, {}, {}",
            QualifierStrategy::None.as_str(),
            QualifierStrategy::AnySuffix.as_str(),
            QualifierStrategy::Allowlist.as_str(),
        ))
    })
}

fn expect_optional_string_list(
    table: &Table,
    key: &str,
    location: &str,
    require_unique: bool,
) -> Result<Option<Vec<String>>, ConfigError> {
    let Some(value) = table.get(key) else {
        return Ok(None);
    };

    let Some(array) = value.as_array() else {
        return Err(ConfigError::validation(format!(
            "{location}.{key} must be an array of strings"
        )));
    };

    let mut items = Vec::with_capacity(array.len());
    let mut seen_indices: BTreeMap<String, usize> = BTreeMap::new();
    for (index, item) in array.iter().enumerate() {
        let Some(item) = item.as_str() else {
            return Err(ConfigError::validation(format!(
                "{location}.{key}[{index}] must be a non-empty string"
            )));
        };
        if item.trim().is_empty() {
            return Err(ConfigError::validation(format!(
                "{location}.{key}[{index}] must be a non-empty string"
            )));
        }

        if require_unique {
            if let Some(first_index) = seen_indices.get(item) {
                return Err(ConfigError::validation(format!(
                    "{location}.{key} contains duplicate value {item:?} at indices {first_index} and {index}"
                )));
            }
            seen_indices.insert(item.to_owned(), index);
        }

        items.push(item.to_owned());
    }

    Ok(Some(items))
}

fn raise_on_duplicate_strings(values: &[String], location: &str) -> Result<(), ConfigError> {
    let mut seen_indices: BTreeMap<&str, usize> = BTreeMap::new();
    for (index, value) in values.iter().enumerate() {
        if let Some(first_index) = seen_indices.get(value.as_str()) {
            return Err(ConfigError::validation(format!(
                "{location} contains duplicate value {value:?} at indices {first_index} and {index}"
            )));
        }
        seen_indices.insert(value.as_str(), index);
    }
    Ok(())
}

fn expect_optional_rule_ids(
    table: &Table,
    key: &str,
    location: &str,
) -> Result<Option<Vec<RuleId>>, ConfigError> {
    let Some(values) = expect_optional_string_list(table, key, location, true)? else {
        return Ok(None);
    };

    let mut rule_ids = Vec::with_capacity(values.len());
    for (index, value) in values.iter().enumerate() {
        let parsed = RuleId::parse(value).map_err(|_| {
            ConfigError::validation(format!(
                "{location}.{key}[{index}] contains invalid rule id: {value}"
            ))
        })?;
        rule_ids.push(parsed);
    }
    Ok(Some(rule_ids))
}

fn expect_optional_targets(
    table: &Table,
    key: &str,
) -> Result<Option<Vec<PartialTargetConfig>>, ConfigError> {
    let Some(value) = table.get(key) else {
        return Ok(None);
    };

    let Some(array) = value.as_array() else {
        return Err(ConfigError::validation(
            "tool.tq.targets must be an array of tables",
        ));
    };

    let mut targets = Vec::with_capacity(array.len());
    for (target_index, item) in array.iter().enumerate() {
        let location = format!("tool.tq.targets[{target_index}]");
        let Some(target_table) = item.as_table() else {
            return Err(ConfigError::validation(format!(
                "{location} must be a table"
            )));
        };

        reject_unknown_keys(
            target_table,
            &[
                "name",
                "package",
                "source_root",
                "test_root",
                "ignore_init_modules",
                "max_test_file_non_blank_lines",
                "qualifier_strategy",
                "allowed_qualifiers",
                "select",
                "ignore",
            ],
            &format!("Unknown key(s) in {location}:"),
        )?;

        targets.push(PartialTargetConfig {
            name: expect_optional_str(target_table, "name", &location)?,
            package: expect_optional_str(target_table, "package", &location)?,
            source_root: expect_optional_str(target_table, "source_root", &location)?,
            test_root: expect_optional_str(target_table, "test_root", &location)?,
            ignore_init_modules: expect_optional_bool(
                target_table,
                "ignore_init_modules",
                &location,
            )?,
            max_test_file_non_blank_lines: expect_optional_positive_int(
                target_table,
                "max_test_file_non_blank_lines",
                &location,
            )?,
            qualifier_strategy: expect_optional_qualifier_strategy(
                target_table,
                "qualifier_strategy",
                &location,
            )?,
            allowed_qualifiers: expect_optional_string_list(
                target_table,
                "allowed_qualifiers",
                &location,
                true,
            )?,
            select: expect_optional_rule_ids(target_table, "select", &location)?,
            ignore: expect_optional_rule_ids(target_table, "ignore", &location)?,
        });
    }

    Ok(Some(targets))
}
