use std::path::{Path, PathBuf};

use serde::Deserialize;

use crate::ReleaseError;

const DEPENDABOT_CONFIG_PATH: &str = ".github/dependabot.yml";
const GITHUB_ACTIONS_ECOSYSTEM: &str = "github-actions";
const ACTIONS_ROOT: &str = ".github/actions";
const WORKFLOWS_ROOT: &str = ".github/workflows";
const REQUIRED_WORKFLOW_PATTERN: &str = "/";
const REQUIRED_ACTIONS_PATTERN: &str = "/.github/actions/*";

#[derive(Debug, Deserialize)]
struct DependabotConfig {
    #[serde(default)]
    updates: Vec<DependabotUpdate>,
}

#[derive(Debug, Deserialize)]
struct DependabotUpdate {
    #[serde(rename = "package-ecosystem")]
    package_ecosystem: String,
    directory: Option<String>,
    #[serde(default)]
    directories: Vec<String>,
}

pub fn verify_dependabot(repo_root: &Path) -> Result<(), ReleaseError> {
    let config_path = repo_root.join(DEPENDABOT_CONFIG_PATH);
    let config_contents =
        std::fs::read_to_string(&config_path).map_err(|source| ReleaseError::Io {
            path: config_path.clone(),
            source,
        })?;
    let config: DependabotConfig =
        serde_yaml::from_str(&config_contents).map_err(|source| ReleaseError::Yaml {
            path: config_path.clone(),
            source,
        })?;

    let github_actions_updates = config
        .updates
        .into_iter()
        .filter(|update| update.package_ecosystem == GITHUB_ACTIONS_ECOSYSTEM)
        .collect::<Vec<_>>();

    let mut violations = Vec::new();
    if github_actions_updates.len() != 1 {
        violations.push(format!(
            "expected exactly one github-actions update block in {}",
            config_path.display()
        ));
    }

    let configured_patterns = collect_directory_patterns(&github_actions_updates);
    let local_action_directories = local_action_directories(repo_root)?;
    let uncovered_directories =
        uncovered_directories(&local_action_directories, &configured_patterns);
    if !uncovered_directories.is_empty() {
        violations.push(format!(
            "github-actions Dependabot config does not cover local action directories: {}",
            uncovered_directories.join(", ")
        ));
    }

    let workflow_files = local_workflow_files(repo_root)?;
    if !workflow_files.is_empty()
        && !configured_patterns
            .iter()
            .any(|pattern| pattern == REQUIRED_WORKFLOW_PATTERN)
    {
        violations.push(format!(
            "github-actions Dependabot config must include directory pattern {REQUIRED_WORKFLOW_PATTERN:?} to cover .github/workflows"
        ));
    }

    if !configured_patterns
        .iter()
        .any(|pattern| pattern == REQUIRED_ACTIONS_PATTERN)
    {
        violations.push(format!(
            "github-actions Dependabot config must include directory pattern {REQUIRED_ACTIONS_PATTERN:?}"
        ));
    }

    if violations.is_empty() {
        return Ok(());
    }

    Err(ReleaseError::RepositoryPolicyViolation {
        details: violations.join("\n"),
    })
}

fn collect_directory_patterns(updates: &[DependabotUpdate]) -> Vec<String> {
    let mut patterns = Vec::new();

    for update in updates {
        if let Some(directory) = &update.directory {
            patterns.push(directory.clone());
        }

        patterns.extend(update.directories.iter().cloned());
    }

    patterns
}

fn local_action_directories(repo_root: &Path) -> Result<Vec<String>, ReleaseError> {
    let actions_root = repo_root.join(ACTIONS_ROOT);
    if !actions_root.exists() {
        return Ok(Vec::new());
    }

    let mut directories = Vec::new();
    collect_action_directories(repo_root, &actions_root, &mut directories)?;
    directories.sort();
    directories.dedup();
    Ok(directories)
}

fn collect_action_directories(
    repo_root: &Path,
    current_dir: &Path,
    directories: &mut Vec<String>,
) -> Result<(), ReleaseError> {
    for entry in std::fs::read_dir(current_dir).map_err(|source| ReleaseError::Io {
        path: current_dir.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| ReleaseError::Io {
            path: current_dir.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        if path.is_dir() {
            collect_action_directories(repo_root, &path, directories)?;
            continue;
        }

        let Some(file_name) = path.file_name().and_then(std::ffi::OsStr::to_str) else {
            continue;
        };
        if file_name != "action.yml" && file_name != "action.yaml" {
            continue;
        }

        let relative_parent = path
            .parent()
            .and_then(|parent| parent.strip_prefix(repo_root).ok())
            .map_or_else(PathBuf::new, Path::to_path_buf);
        let relative_text = relative_parent
            .as_os_str()
            .to_string_lossy()
            .replace('\\', "/");
        directories.push(format!("/{relative_text}"));
    }

    Ok(())
}

fn local_workflow_files(repo_root: &Path) -> Result<Vec<PathBuf>, ReleaseError> {
    let workflows_root = repo_root.join(WORKFLOWS_ROOT);
    if !workflows_root.exists() {
        return Ok(Vec::new());
    }

    let mut files = std::fs::read_dir(&workflows_root)
        .map_err(|source| ReleaseError::Io {
            path: workflows_root.clone(),
            source,
        })?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| {
            path.extension()
                .and_then(std::ffi::OsStr::to_str)
                .is_some_and(|extension| extension == "yml" || extension == "yaml")
        })
        .collect::<Vec<_>>();
    files.sort();
    Ok(files)
}

fn uncovered_directories(directories: &[String], configured_patterns: &[String]) -> Vec<String> {
    let mut uncovered = directories
        .iter()
        .filter(|directory| {
            !configured_patterns
                .iter()
                .any(|pattern| pattern_matches(directory, pattern))
        })
        .cloned()
        .collect::<Vec<_>>();
    uncovered.sort();
    uncovered
}

fn pattern_matches(value: &str, pattern: &str) -> bool {
    wildcard_match(value.as_bytes(), pattern.as_bytes())
}

fn wildcard_match(value: &[u8], pattern: &[u8]) -> bool {
    let (mut value_index, mut pattern_index) = (0usize, 0usize);
    let (mut star_pattern_index, mut star_value_index) = (None, 0usize);

    while value_index < value.len() {
        if pattern_index < pattern.len()
            && (pattern[pattern_index] == value[value_index] || pattern[pattern_index] == b'?')
        {
            value_index += 1;
            pattern_index += 1;
            continue;
        }

        if pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
            star_pattern_index = Some(pattern_index);
            pattern_index += 1;
            star_value_index = value_index;
            continue;
        }

        let Some(saved_pattern_index) = star_pattern_index else {
            return false;
        };
        pattern_index = saved_pattern_index + 1;
        star_value_index += 1;
        value_index = star_value_index;
    }

    while pattern_index < pattern.len() && pattern[pattern_index] == b'*' {
        pattern_index += 1;
    }

    pattern_index == pattern.len()
}
