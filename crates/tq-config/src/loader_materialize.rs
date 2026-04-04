use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

use tq_core::{PackageName, QualifierStrategy, TargetName};

use crate::{
    ConfigError, DEFAULT_INIT_MODULES, DEFAULT_MAX_TEST_FILE_NON_BLANK_LINES,
    model::{PartialRuleConfig, PartialTargetConfig, PartialTqConfig, TqConfig, TqTargetConfig},
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
        return Err(ConfigError::MissingTargets);
    };
    if targets.is_empty() {
        return Err(ConfigError::MissingTargets);
    }

    let base_dir = targets_base_dir.map_or_else(|| cwd.to_path_buf(), PathBuf::from);

    let mut normalized_targets = Vec::with_capacity(targets.len());
    let mut seen_names: BTreeMap<TargetName, usize> = BTreeMap::new();
    let mut seen_roots: BTreeMap<PathBuf, usize> = BTreeMap::new();

    for (target_index, target) in targets.iter().enumerate() {
        let resolved = materialize_target(
            &base_dir,
            target,
            &partial.defaults,
            cli_defaults,
            target_index,
        )?;

        if let Some(first_index) = seen_names.get(resolved.name()) {
            return Err(ConfigError::DuplicateTargetName {
                first_index: *first_index,
                second_index: target_index,
                name: resolved.name().to_string(),
            });
        }
        seen_names.insert(resolved.name().clone(), target_index);

        let source_package_root = source_package_root_key(&resolved);
        if let Some(first_index) = seen_roots.get(&source_package_root) {
            return Err(ConfigError::DuplicateSourcePackageRoot {
                first_index: *first_index,
                second_index: target_index,
                path: source_package_root.clone(),
            });
        }
        seen_roots.insert(source_package_root, target_index);

        normalized_targets.push(resolved);
    }

    normalized_targets.sort_by(|left, right| left.name().cmp(right.name()));
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

    let name_value = require_target_key(target.name.as_ref(), "name", &location)?;
    let name = TargetName::parse(&name_value).map_err(|source| ConfigError::InvalidTargetName {
        location: format!("{location}.name"),
        value: name_value.clone(),
        source,
    })?;

    let package_value = require_target_key(target.package.as_ref(), "package", &location)?;
    let package =
        PackageName::parse(&package_value).map_err(|source| ConfigError::InvalidPackageName {
            location: format!("{location}.package"),
            value: package_value.clone(),
            source,
        })?;

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
        return Err(ConfigError::AllowlistRequiresQualifiers {
            location: format!("{location}.allowed_qualifiers"),
        });
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
        return Err(ConfigError::MissingTargetKey {
            location: format!("{location}.{key}"),
        });
    };
    if value.trim().is_empty() {
        return Err(ConfigError::EmptyString {
            location: format!("{location}.{key}"),
        });
    }
    Ok(value.clone())
}

fn resolve_path(base_dir: &Path, value: &str) -> PathBuf {
    let candidate = Path::new(value);
    if candidate.is_absolute() {
        return normalize_absolute(candidate);
    }
    normalize_absolute(&base_dir.join(candidate))
}
