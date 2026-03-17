use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::Path;

use toml::Table;

use crate::{
    ConfigError, InitModulesMode, PartialRuleConfig, PartialTargetConfig, PartialTqConfig,
    QualifierStrategy, RuleId,
};

pub fn load_partial_from_pyproject(
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
            "init_modules",
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
        init_modules: expect_optional_init_modules(tq_table, "init_modules", "tool.tq")?,
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

pub fn ensure_unique_strings(values: &[String], location: &str) -> Result<(), ConfigError> {
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

fn expect_optional_init_modules(
    table: &Table,
    key: &str,
    location: &str,
) -> Result<Option<InitModulesMode>, ConfigError> {
    let Some(value) = table.get(key) else {
        return Ok(None);
    };

    let Some(raw) = value.as_str() else {
        return Err(ConfigError::validation(format!(
            "{location}.{key} must be a string"
        )));
    };

    InitModulesMode::parse(raw).map(Some).ok_or_else(|| {
        ConfigError::validation(format!(
            "{location}.{key} must be one of: {}, {}",
            InitModulesMode::Include.as_str(),
            InitModulesMode::Ignore.as_str(),
        ))
    })
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
                "init_modules",
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
            init_modules: expect_optional_init_modules(target_table, "init_modules", &location)?,
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
