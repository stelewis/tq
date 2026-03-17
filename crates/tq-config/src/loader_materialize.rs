use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use crate::{
    ConfigError, DEFAULT_INIT_MODULES, DEFAULT_MAX_TEST_FILE_NON_BLANK_LINES, PartialRuleConfig,
    PartialTargetConfig, PartialTqConfig, QualifierStrategy, TqConfig, TqTargetConfig,
    paths::normalize_absolute,
};

pub fn merge_partial(base: &PartialTqConfig, override_: &PartialTqConfig) -> PartialTqConfig {
    PartialTqConfig {
        defaults: merge_rule_partial(&base.defaults, &override_.defaults),
        targets: override_.targets.clone().or_else(|| base.targets.clone()),
    }
}

pub fn materialize_config(
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

        let source_package_root = source_package_root_key(&resolved);
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

fn merge_rule_partial(
    base: &PartialRuleConfig,
    override_: &PartialRuleConfig,
) -> PartialRuleConfig {
    PartialRuleConfig {
        init_modules: override_.init_modules.or(base.init_modules),
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

fn source_package_root_key(target: &TqTargetConfig) -> PathBuf {
    let lexical = target.source_package_root();
    fs::canonicalize(&lexical).unwrap_or(lexical)
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
            init_modules: target.init_modules,
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
        init_modules: final_rules.init_modules.unwrap_or(DEFAULT_INIT_MODULES),
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
