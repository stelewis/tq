use std::path::{Path, PathBuf};

use crate::ReleaseError;

const DEPENDABOT_CONFIG_PATH: &str = ".github/dependabot.yml";
const GITHUB_ACTIONS_ECOSYSTEM: &str = "github-actions";
const ACTIONS_ROOT: &str = ".github/actions";
const WORKFLOWS_ROOT: &str = ".github/workflows";
const REQUIRED_WORKFLOW_PATTERN: &str = "/";
const REQUIRED_ACTIONS_PATTERN: &str = "/.github/actions/*";

#[derive(Debug)]
struct DependabotUpdate {
    package_ecosystem: String,
    directory: Option<String>,
    directories: Vec<String>,
}

pub fn verify_dependabot(repo_root: &Path) -> Result<(), ReleaseError> {
    let config_path = repo_root.join(DEPENDABOT_CONFIG_PATH);
    let config_contents =
        std::fs::read_to_string(&config_path).map_err(|source| ReleaseError::Io {
            path: config_path.clone(),
            source,
        })?;
    let config = parse_dependabot_updates(&config_contents).map_err(|message| {
        ReleaseError::DependabotConfig {
            path: config_path.clone(),
            message,
        }
    })?;

    let github_actions_updates = config
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

fn parse_dependabot_updates(input: &str) -> Result<Vec<DependabotUpdate>, String> {
    let mut parser = DependabotParser::default();

    for (line_index, raw_line) in input.lines().enumerate() {
        let line_number = line_index + 1;
        let trimmed = raw_line.trim();
        if trimmed.is_empty() || trimmed.starts_with('#') {
            continue;
        }

        let indent = raw_line
            .chars()
            .take_while(|character| *character == ' ')
            .count();
        if indent == raw_line.len() {
            continue;
        }

        if parser.skip_ignored_block(indent) {
            continue;
        }

        parser.parse_line(trimmed, indent, line_number)?;
    }

    parser.finish()
}

#[derive(Debug, Default)]
struct DependabotParser {
    version: Option<String>,
    updates_section_found: bool,
    updates: Vec<DependabotUpdate>,
    current_update: Option<PendingUpdate>,
    active_list: ActiveList,
    ignored_block_indent: Option<usize>,
}

impl DependabotParser {
    const fn skip_ignored_block(&mut self, indent: usize) -> bool {
        if let Some(skip_indent) = self.ignored_block_indent {
            if indent > skip_indent {
                return true;
            }
            self.ignored_block_indent = None;
        }

        false
    }

    fn parse_line(
        &mut self,
        trimmed: &str,
        indent: usize,
        line_number: usize,
    ) -> Result<(), String> {
        if indent == 0 {
            return self.parse_top_level_line(trimmed, line_number);
        }

        if !self.updates_section_found {
            return Err(format!(
                "line {line_number}: unexpected nested content before updates section"
            ));
        }

        self.parse_update_line(trimmed, indent, line_number)
    }

    fn parse_top_level_line(&mut self, trimmed: &str, line_number: usize) -> Result<(), String> {
        self.finish_current_update()?;
        self.active_list = ActiveList::None;

        let (key, value) = split_key_value(trimmed)
            .ok_or_else(|| format!("line {line_number}: expected top-level key"))?;
        match key {
            "version" => self.parse_version(value, line_number),
            "updates" => self.parse_updates_section(value, line_number),
            "registries" | "multi-ecosystem-groups" => {
                if !value.is_empty() {
                    return Err(format!(
                        "line {line_number}: {key} must be declared as a block"
                    ));
                }
                self.ignored_block_indent = Some(0);
                Ok(())
            }
            _ => Err(format!("line {line_number}: unknown top-level key {key}")),
        }
    }

    fn parse_version(&mut self, value: &str, line_number: usize) -> Result<(), String> {
        if self.version.is_some() {
            return Err(format!(
                "line {line_number}: duplicate top-level key version"
            ));
        }

        self.version = Some(parse_scalar(value, line_number)?);
        Ok(())
    }

    fn parse_updates_section(&mut self, value: &str, line_number: usize) -> Result<(), String> {
        if self.updates_section_found {
            return Err(format!(
                "line {line_number}: duplicate top-level key updates"
            ));
        }
        if !value.is_empty() {
            return Err(format!(
                "line {line_number}: updates must be declared as a block"
            ));
        }

        self.updates_section_found = true;
        Ok(())
    }

    fn parse_update_line(
        &mut self,
        trimmed: &str,
        indent: usize,
        line_number: usize,
    ) -> Result<(), String> {
        match indent {
            2 => self.parse_update_item(trimmed, line_number),
            4 => self.parse_update_key_line(trimmed, line_number),
            6 if trimmed.starts_with("- ") => self.parse_directories_item(trimmed, line_number),
            _ if self.active_list == ActiveList::Directories => Err(format!(
                "line {line_number}: directories entries must be list items at indent 6"
            )),
            _ => Err(format!(
                "line {line_number}: unsupported update structure or indentation"
            )),
        }
    }

    fn parse_update_item(&mut self, trimmed: &str, line_number: usize) -> Result<(), String> {
        if !trimmed.starts_with("- ") {
            return Err(format!(
                "line {line_number}: updates entries must start with '- ' at indent 2"
            ));
        }

        self.finish_current_update()?;

        let remainder = trimmed.trim_start_matches("- ").trim();
        self.current_update = Some(PendingUpdate::default());
        self.active_list = ActiveList::None;
        if remainder.is_empty() {
            return Ok(());
        }

        let (key, value) = split_key_value(remainder).ok_or_else(|| {
            format!("line {line_number}: expected key/value after update list item marker")
        })?;
        self.apply_directive(key, value, line_number, 2)
    }

    fn parse_update_key_line(&mut self, trimmed: &str, line_number: usize) -> Result<(), String> {
        self.active_list = ActiveList::None;
        let (key, value) = split_key_value(trimmed)
            .ok_or_else(|| format!("line {line_number}: expected update key/value pair"))?;
        self.apply_directive(key, value, line_number, 4)
    }

    fn parse_directories_item(&mut self, trimmed: &str, line_number: usize) -> Result<(), String> {
        if self.active_list != ActiveList::Directories {
            return Err(format!(
                "line {line_number}: unexpected list item outside directories block"
            ));
        }

        let update = self.current_update_mut(line_number)?;
        update.directories.push(parse_scalar(
            trimmed.trim_start_matches("- ").trim(),
            line_number,
        )?);
        Ok(())
    }

    fn apply_directive(
        &mut self,
        key: &str,
        value: &str,
        line_number: usize,
        indent: usize,
    ) -> Result<(), String> {
        let directive = apply_update_key(
            self.current_update_mut(line_number)?,
            key,
            value,
            line_number,
        )?;
        match directive {
            UpdateDirective::None => {}
            UpdateDirective::Directories => self.active_list = ActiveList::Directories,
            UpdateDirective::IgnoreNestedBlock => self.ignored_block_indent = Some(indent),
        }
        Ok(())
    }

    fn current_update_mut(&mut self, line_number: usize) -> Result<&mut PendingUpdate, String> {
        self.current_update.as_mut().ok_or_else(|| {
            format!("line {line_number}: updates entries must start with '- ' at indent 2")
        })
    }

    fn finish_current_update(&mut self) -> Result<(), String> {
        if let Some(update) = self.current_update.take() {
            self.updates.push(update.finish()?);
        }
        Ok(())
    }

    fn finish(mut self) -> Result<Vec<DependabotUpdate>, String> {
        self.finish_current_update()?;

        match self.version.as_deref() {
            Some("2") => {}
            Some(other) => return Err(format!("unsupported Dependabot version: {other}")),
            None => return Err("missing Dependabot version".to_owned()),
        }

        if !self.updates_section_found {
            return Err("missing updates section".to_owned());
        }

        Ok(self.updates)
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
enum ActiveList {
    #[default]
    None,
    Directories,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum UpdateDirective {
    None,
    Directories,
    IgnoreNestedBlock,
}

#[derive(Debug, Default)]
struct PendingUpdate {
    package_ecosystem: Option<String>,
    directory: Option<String>,
    directories_declared: bool,
    directories: Vec<String>,
}

impl PendingUpdate {
    fn finish(self) -> Result<DependabotUpdate, String> {
        let package_ecosystem = self
            .package_ecosystem
            .ok_or_else(|| "dependabot update is missing package-ecosystem".to_owned())?;

        if self.directory.is_some() == self.directories_declared {
            return Err(
                "dependabot update must define exactly one of directory or directories".to_owned(),
            );
        }

        if self.directories_declared && self.directories.is_empty() {
            return Err("dependabot update directories must contain at least one entry".to_owned());
        }

        Ok(DependabotUpdate {
            package_ecosystem,
            directory: self.directory,
            directories: self.directories,
        })
    }
}

fn apply_update_key(
    update: &mut PendingUpdate,
    key: &str,
    value: &str,
    line_number: usize,
) -> Result<UpdateDirective, String> {
    match key {
        "package-ecosystem" => {
            update.package_ecosystem = Some(parse_scalar(value, line_number)?);
            Ok(UpdateDirective::None)
        }
        "directory" => {
            update.directory = Some(parse_scalar(value, line_number)?);
            Ok(UpdateDirective::None)
        }
        "directories" => {
            if !value.is_empty() {
                return Err(format!(
                    "line {line_number}: directories must be declared as a block list"
                ));
            }
            update.directories_declared = true;
            Ok(UpdateDirective::Directories)
        }
        "schedule"
        | "commit-message"
        | "allow"
        | "ignore"
        | "labels"
        | "assignees"
        | "reviewers"
        | "registries"
        | "groups"
        | "cooldown"
        | "milestone"
        | "target-branch"
        | "open-pull-requests-limit"
        | "pull-request-branch-name"
        | "rebase-strategy"
        | "insecure-external-code-execution"
        | "vendor" => {
            if value.is_empty() {
                return Ok(UpdateDirective::IgnoreNestedBlock);
            }

            parse_scalar(value, line_number)?;
            Ok(UpdateDirective::None)
        }
        _ => Err(format!("line {line_number}: unknown update key {key}")),
    }
}

fn split_key_value(line: &str) -> Option<(&str, &str)> {
    let (key, value) = line.split_once(':')?;
    Some((key.trim(), value.trim()))
}

fn parse_scalar(value: &str, line_number: usize) -> Result<String, String> {
    if value.is_empty() {
        return Err(format!("line {line_number}: expected scalar value"));
    }

    if let Some(unquoted) = strip_wrapping_quotes(value) {
        return Ok(unquoted.to_owned());
    }

    Ok(value.to_owned())
}

fn strip_wrapping_quotes(value: &str) -> Option<&str> {
    if value.len() < 2 {
        return None;
    }

    let bytes = value.as_bytes();
    let first = bytes.first()?;
    let last = bytes.last()?;
    if (*first == b'"' && *last == b'"') || (*first == b'\'' && *last == b'\'') {
        return Some(&value[1..value.len() - 1]);
    }

    None
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
