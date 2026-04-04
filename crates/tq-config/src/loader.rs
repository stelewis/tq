use std::path::{Path, PathBuf};

use crate::{
    ConfigError,
    loader_materialize::{materialize_config, merge_partial},
    loader_parse::{ensure_unique_strings, load_partial_from_pyproject},
    model::{CliOverrides, PartialRuleConfig, PartialTqConfig, TqConfig},
    paths::normalize_absolute,
};

pub fn resolve_tq_config(
    cwd: &Path,
    explicit_config_path: Option<&Path>,
    isolated: bool,
    cli_overrides: &CliOverrides,
) -> Result<TqConfig, ConfigError> {
    resolve_tq_config_with_user_config(cwd, explicit_config_path, isolated, None, cli_overrides)
}

pub fn resolve_tq_config_with_user_config(
    cwd: &Path,
    explicit_config_path: Option<&Path>,
    isolated: bool,
    discovered_user_config_path: Option<&Path>,
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
        let user_config_path = discovered_user_config_path
            .map(Path::to_path_buf)
            .or_else(|| {
                home_dir().map(|home| home.join(".config").join("tq").join("pyproject.toml"))
            });
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

    let current = std::env::current_dir().map_err(|error| ConfigError::CurrentDirectory {
        message: error.to_string(),
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
        },
        targets: None,
    })
}
