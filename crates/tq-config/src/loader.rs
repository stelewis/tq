use std::path::{Path, PathBuf};

use crate::{
    ConfigError,
    loader_materialize::{materialize_config, merge_partial},
    loader_parse::{SectionPolicy, ensure_unique_strings, load_partial_from_pyproject},
    model::{CliOverrides, PartialRuleConfig, PartialTqConfig, TqConfig},
    paths::normalize_absolute,
};

/// Resolves the effective configuration from explicit, user, and project
/// pyproject sources plus CLI overrides.
///
/// The loader performs no ambient environment access: `cwd` must be an
/// absolute path and `user_config_path` (typically under the user's home
/// configuration directory) is supplied by the composition root.
pub fn resolve_tq_config(
    cwd: &Path,
    explicit_config_path: Option<&Path>,
    isolated: bool,
    user_config_path: Option<&Path>,
    cli_overrides: &CliOverrides,
) -> Result<TqConfig, ConfigError> {
    let cwd = require_absolute(cwd)?;
    let mut discovered = PartialTqConfig::default();
    let mut targets_base_dir: Option<PathBuf> = None;

    if let Some(explicit_path) = explicit_config_path {
        let config_path = absolute_from(&cwd, explicit_path);
        let loaded = load_partial_from_pyproject(&config_path, SectionPolicy::Required)?;
        targets_base_dir = resolve_targets_base_dir(targets_base_dir, &loaded, &config_path);
        discovered = loaded;
    } else if !isolated {
        if let Some(user_config_path) = user_config_path.filter(|path| path.exists()) {
            let user_config_path = absolute_from(&cwd, user_config_path);
            let user_partial =
                load_partial_from_pyproject(&user_config_path, SectionPolicy::Optional)?;
            targets_base_dir =
                resolve_targets_base_dir(targets_base_dir, &user_partial, &user_config_path);
            discovered = merge_partial(&discovered, &user_partial);
        }

        if let Some(project_config_path) = find_project_pyproject(&cwd) {
            let project_partial =
                load_partial_from_pyproject(&project_config_path, SectionPolicy::Optional)?;
            targets_base_dir =
                resolve_targets_base_dir(targets_base_dir, &project_partial, &project_config_path);
            discovered = merge_partial(&discovered, &project_partial);
        }
    }

    let cli_partial = partial_from_cli(cli_overrides)?;
    materialize_config(&cwd, &discovered, &cli_partial, targets_base_dir.as_deref())
}

fn require_absolute(cwd: &Path) -> Result<PathBuf, ConfigError> {
    if cwd.is_absolute() {
        Ok(normalize_absolute(cwd))
    } else {
        Err(ConfigError::RelativeWorkingDirectory {
            path: cwd.to_path_buf(),
        })
    }
}

fn absolute_from(cwd: &Path, path: &Path) -> PathBuf {
    if path.is_absolute() {
        normalize_absolute(path)
    } else {
        normalize_absolute(&cwd.join(path))
    }
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

fn partial_from_cli(overrides: &CliOverrides) -> Result<PartialTqConfig, ConfigError> {
    if let Some(values) = overrides.allowed_qualifiers() {
        ensure_unique_strings(values, "cli.allowed_qualifiers")?;
    }

    Ok(PartialTqConfig {
        defaults: PartialRuleConfig {
            init_modules: overrides.init_modules(),
            max_test_file_non_blank_lines: overrides.max_test_file_non_blank_lines(),
            qualifier_strategy: overrides.qualifier_strategy(),
            allowed_qualifiers: overrides.clone_allowed_qualifiers(),
            select: overrides.clone_select(),
            ignore: overrides.clone_ignore(),
            severity_overrides: overrides.clone_severity_overrides(),
        },
        targets: None,
        fail_on: overrides.fail_on(),
    })
}
