//! Strict readers for the repository-owned TOML, JSON, and YAML surfaces the
//! harness verifies.
//!
//! YAML handling is deliberately line-oriented and limited to the shapes this
//! repository commits. See ADR 0003 for the supply-chain rationale.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use toml::Value;

use crate::error::DevError;

pub fn read_to_string(path: &Path) -> Result<String, DevError> {
    std::fs::read_to_string(path).map_err(|source| DevError::Io {
        path: path.to_path_buf(),
        source,
    })
}

pub fn read_toml(path: &Path) -> Result<Value, DevError> {
    let contents = read_to_string(path)?;
    contents
        .parse::<toml::Table>()
        .map(Value::Table)
        .map_err(|source| DevError::InvalidInput {
            path: path.to_path_buf(),
            message: source.to_string(),
        })
}

pub fn read_json(path: &Path) -> Result<serde_json::Value, DevError> {
    let contents = read_to_string(path)?;
    serde_json::from_str(&contents).map_err(|source| DevError::InvalidInput {
        path: path.to_path_buf(),
        message: source.to_string(),
    })
}

pub fn required_string(document: &Value, path: &[&str], source: &Path) -> Result<String, DevError> {
    required_value(document, path, source)?
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| invalid_type(source, path, "string"))
}

pub fn required_integer(document: &Value, path: &[&str], source: &Path) -> Result<i64, DevError> {
    required_value(document, path, source)?
        .as_integer()
        .ok_or_else(|| invalid_type(source, path, "integer"))
}

pub fn required_string_array(
    document: &Value,
    path: &[&str],
    source: &Path,
) -> Result<Vec<String>, DevError> {
    required_value(document, path, source)?
        .as_array()
        .ok_or_else(|| invalid_type(source, path, "array"))?
        .iter()
        .map(|value| {
            value
                .as_str()
                .map(ToOwned::to_owned)
                .ok_or_else(|| invalid_type(source, path, "string array"))
        })
        .collect()
}

pub fn required_json_string(
    document: &serde_json::Value,
    path: &[&str],
    source: &Path,
) -> Result<String, DevError> {
    let mut current = document;
    for segment in path {
        current = current
            .get(*segment)
            .ok_or_else(|| missing_value(source, path))?;
    }
    current
        .as_str()
        .map(ToOwned::to_owned)
        .ok_or_else(|| invalid_type(source, path, "string"))
}

fn required_value<'a>(
    document: &'a Value,
    path: &[&str],
    source: &Path,
) -> Result<&'a Value, DevError> {
    let mut current = document;
    for segment in path {
        current = current
            .get(*segment)
            .ok_or_else(|| missing_value(source, path))?;
    }
    Ok(current)
}

fn invalid_type(source: &Path, path: &[&str], expected: &str) -> DevError {
    DevError::InvalidInput {
        path: source.to_path_buf(),
        message: format!("{} must be a {expected}", path.join(".")),
    }
}

fn missing_value(source: &Path, path: &[&str]) -> DevError {
    DevError::InvalidInput {
        path: source.to_path_buf(),
        message: format!("missing {}", path.join(".")),
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ReferenceSource {
    pub path: PathBuf,
    pub line: usize,
}

impl ReferenceSource {
    #[must_use]
    pub fn display(&self) -> String {
        format!("{}:{}", self.path.display(), self.line)
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct CommitSha(String);

impl CommitSha {
    fn parse(value: &str) -> Option<Self> {
        (value.len() == 40
            && value
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte)))
        .then(|| Self(value.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum PinnedRevision {
    Missing,
    Commit(CommitSha),
    Unfrozen(String),
}

impl PinnedRevision {
    fn parse(value: Option<&str>) -> Self {
        value.map_or(Self::Missing, |value| {
            CommitSha::parse(value).map_or_else(|| Self::Unfrozen(value.to_owned()), Self::Commit)
        })
    }

    #[must_use]
    pub fn raw(&self) -> Option<&str> {
        match self {
            Self::Missing => None,
            Self::Commit(sha) => Some(sha.as_str()),
            Self::Unfrozen(value) => Some(value),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub enum ActionReference {
    Local {
        source: ReferenceSource,
        reference: String,
    },
    Docker {
        source: ReferenceSource,
        reference: String,
    },
    External {
        source: ReferenceSource,
        name: String,
        revision: PinnedRevision,
    },
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PreCommitRepository {
    pub source: ReferenceSource,
    pub repository: String,
    pub revision: PinnedRevision,
}

impl PreCommitRepository {
    #[must_use]
    pub fn is_local(&self) -> bool {
        self.repository == "local"
    }
}

pub fn action_references(contents: &str, source: &Path) -> Result<Vec<ActionReference>, DevError> {
    let mut references = Vec::new();
    for (index, line) in contents.lines().enumerate() {
        let parsed = yaml_key_value(line);
        if yaml_mentions_key(line, "uses") && parsed.is_none_or(|(_, key, _)| key != "uses") {
            return Err(DevError::InvalidInput {
                path: source.to_path_buf(),
                message: format!(
                    "line {} uses an unsupported YAML shape for a uses reference",
                    index + 1
                ),
            });
        }
        let Some((_, key, value)) = parsed else {
            continue;
        };
        if key != "uses" {
            continue;
        }
        let reference = yaml_scalar(value);
        let reference_source = ReferenceSource {
            path: source.to_path_buf(),
            line: index + 1,
        };
        if reference.is_empty() {
            return Err(DevError::InvalidInput {
                path: source.to_path_buf(),
                message: format!("line {} has an empty uses reference", index + 1),
            });
        }
        if reference.starts_with("./") {
            references.push(ActionReference::Local {
                source: reference_source,
                reference: reference.to_owned(),
            });
            continue;
        }
        if reference.starts_with("docker://") {
            references.push(ActionReference::Docker {
                source: reference_source,
                reference: reference.to_owned(),
            });
            continue;
        }

        let (name, revision) = reference
            .split_once('@')
            .map_or((reference, PinnedRevision::Missing), |(name, revision)| {
                (name, PinnedRevision::parse(Some(revision)))
            });
        if name.is_empty() {
            return Err(DevError::InvalidInput {
                path: source.to_path_buf(),
                message: format!("line {} has an action reference without a name", index + 1),
            });
        }
        references.push(ActionReference::External {
            source: reference_source,
            name: name.to_owned(),
            revision,
        });
    }
    Ok(references)
}

pub fn pre_commit_repositories(
    contents: &str,
    source: &Path,
) -> Result<Vec<PreCommitRepository>, DevError> {
    struct PendingRepository {
        source: ReferenceSource,
        repository: String,
        revision: Option<String>,
    }

    fn finish(repositories: &mut Vec<PreCommitRepository>, pending: PendingRepository) {
        repositories.push(PreCommitRepository {
            source: pending.source,
            repository: pending.repository,
            revision: PinnedRevision::parse(pending.revision.as_deref()),
        });
    }

    let mut repositories = Vec::new();
    let mut pending: Option<PendingRepository> = None;
    for (index, line) in contents.lines().enumerate() {
        let parsed = yaml_key_value(line);
        for key in ["repo", "rev"] {
            if yaml_mentions_key(line, key)
                && parsed.is_none_or(|(_, parsed_key, _)| parsed_key != key)
            {
                return Err(DevError::InvalidInput {
                    path: source.to_path_buf(),
                    message: format!(
                        "line {} uses an unsupported YAML shape for {key}",
                        index + 1
                    ),
                });
            }
        }
        let Some((indent, key, value)) = parsed else {
            continue;
        };
        if indent == 2 && key == "repo" {
            if let Some(previous) = pending.take() {
                finish(&mut repositories, previous);
            }
            let repository = yaml_scalar(value);
            if repository.is_empty() {
                return Err(DevError::InvalidInput {
                    path: source.to_path_buf(),
                    message: format!("line {} has an empty pre-commit repository", index + 1),
                });
            }
            pending = Some(PendingRepository {
                source: ReferenceSource {
                    path: source.to_path_buf(),
                    line: index + 1,
                },
                repository: repository.to_owned(),
                revision: None,
            });
            continue;
        }
        if indent == 4 && key == "rev" {
            let Some(repository) = pending.as_mut() else {
                return Err(DevError::InvalidInput {
                    path: source.to_path_buf(),
                    message: format!("line {} has rev before repo", index + 1),
                });
            };
            if repository.revision.is_some() {
                return Err(DevError::InvalidInput {
                    path: source.to_path_buf(),
                    message: format!("line {} duplicates a repository rev", index + 1),
                });
            }
            repository.revision = Some(yaml_scalar(value).to_owned());
        }
    }
    if let Some(previous) = pending {
        finish(&mut repositories, previous);
    }
    Ok(repositories)
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct ActionInput {
    pub default: Option<String>,
}

/// Parses the `inputs` block of a composite GitHub Action definition.
pub fn action_inputs(
    contents: &str,
    source: &Path,
) -> Result<BTreeMap<String, ActionInput>, DevError> {
    let mut inputs = BTreeMap::new();
    let mut in_inputs = false;
    let mut current_input: Option<String> = None;

    for line in contents.lines() {
        let Some((indent, key, value)) = yaml_key_value(line) else {
            continue;
        };

        if indent == 0 {
            if key == "inputs" {
                in_inputs = true;
                current_input = None;
                continue;
            }
            if in_inputs {
                break;
            }
        }

        if !in_inputs {
            continue;
        }

        match indent {
            2 => {
                inputs.insert(key.to_owned(), ActionInput { default: None });
                current_input = Some(key.to_owned());
            }
            4 if key == "default" => {
                let Some(input) = current_input.as_deref() else {
                    return Err(DevError::InvalidInput {
                        path: source.to_path_buf(),
                        message: "action input default appeared before an input name".to_owned(),
                    });
                };
                let Some(entry) = inputs.get_mut(input) else {
                    return Err(DevError::InvalidInput {
                        path: source.to_path_buf(),
                        message: format!("missing action input {input}"),
                    });
                };
                entry.default = Some(yaml_scalar(value).to_owned());
            }
            _ => {}
        }
    }

    Ok(inputs)
}

pub fn required_action_input_default(
    inputs: &BTreeMap<String, ActionInput>,
    input: &str,
    source: &Path,
) -> Result<String, DevError> {
    inputs
        .get(input)
        .ok_or_else(|| missing_value(source, &["inputs", input]))?
        .default
        .clone()
        .ok_or_else(|| missing_value(source, &["inputs", input, "default"]))
}

/// Finds the `with.<key>` value of the first step whose `uses` reference
/// starts with `uses_prefix`.
pub fn required_action_step_with_value(
    contents: &str,
    uses_prefix: &str,
    with_key: &str,
    source: &Path,
) -> Result<String, DevError> {
    let mut in_target_step = false;
    let mut in_with = false;

    for line in contents.lines() {
        let Some((indent, key, value)) = yaml_key_value(line) else {
            continue;
        };

        if indent == 4 && key == "name" {
            in_target_step = false;
            in_with = false;
            continue;
        }

        if (indent == 4 || indent == 6) && key == "uses" {
            in_target_step = yaml_scalar(value).starts_with(uses_prefix);
            in_with = false;
            continue;
        }

        if in_target_step && indent == 6 && key == "with" {
            in_with = true;
            continue;
        }

        if in_target_step && in_with && indent == 8 && key == with_key {
            return Ok(yaml_scalar(value).to_owned());
        }
    }

    Err(missing_value(
        source,
        &["runs", "steps", uses_prefix, "with", with_key],
    ))
}

fn yaml_mentions_key(line: &str, key: &str) -> bool {
    let without_comment = line.split('#').next().unwrap_or(line);
    without_comment.contains(&format!("{key}:"))
}

pub(crate) fn yaml_key_value(line: &str) -> Option<(usize, &str, &str)> {
    let without_comment = line.split('#').next().unwrap_or(line);
    let trimmed_end = without_comment.trim_end();
    if trimmed_end.trim().is_empty() {
        return None;
    }
    let indent = trimmed_end.len() - trimmed_end.trim_start().len();
    let trimmed = trimmed_end
        .trim_start()
        .strip_prefix("- ")
        .unwrap_or_else(|| trimmed_end.trim_start());
    let (key, value) = trimmed.split_once(':')?;
    Some((indent, key.trim(), value.trim()))
}

#[must_use]
pub fn yaml_scalar(value: &str) -> &str {
    value
        .split('#')
        .next()
        .unwrap_or(value)
        .trim()
        .trim_matches(['\'', '"'])
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        ActionReference, PinnedRevision, action_inputs, action_references, pre_commit_repositories,
        required_action_input_default, required_action_step_with_value,
    };

    #[test]
    fn parses_action_input_defaults_and_step_with_values() {
        let contents = concat!(
            "inputs:\n",
            "  uv-version:\n",
            "    description: uv version\n",
            "    default: '0.11.28'\n",
            "runs:\n",
            "  using: composite\n",
            "  steps:\n",
            "    - name: Set up uv\n",
            "      uses: astral-sh/setup-uv@example\n",
            "      with:\n",
            "        version: ${{ inputs.uv-version }}\n",
        );
        let source = Path::new("action.yml");
        let inputs = action_inputs(contents, source).expect("action inputs should parse");

        assert_eq!(
            required_action_input_default(&inputs, "uv-version", source)
                .expect("uv-version default should exist"),
            "0.11.28"
        );
        assert_eq!(
            required_action_step_with_value(contents, "astral-sh/setup-uv@", "version", source)
                .expect("setup-uv version input should exist"),
            "${{ inputs.uv-version }}"
        );
    }

    #[test]
    fn parses_external_local_and_docker_action_references() {
        let contents = concat!(
            "steps:\n",
            "  - uses: actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0\n",
            "  - uses: ./local-action\n",
            "  - uses: docker://alpine:3.20\n",
        );
        let references = action_references(contents, Path::new("workflow.yml"))
            .expect("action references should parse");

        assert!(matches!(
            &references[0],
            ActionReference::External {
                revision: PinnedRevision::Commit(_),
                ..
            }
        ));
        assert!(matches!(&references[1], ActionReference::Local { .. }));
        assert!(matches!(&references[2], ActionReference::Docker { .. }));
    }

    #[test]
    fn preserves_missing_and_unfrozen_pre_commit_revisions() {
        let contents = concat!(
            "repos:\n",
            "  - repo: https://github.com/example/missing\n",
            "    hooks: []\n",
            "  - repo: https://github.com/example/tagged\n",
            "    rev: v1.2.3\n",
            "    hooks: []\n",
        );
        let repositories = pre_commit_repositories(contents, Path::new(".pre-commit-config.yaml"))
            .expect("pre-commit repositories should parse");

        assert_eq!(repositories[0].revision, PinnedRevision::Missing);
        assert_eq!(
            repositories[1].revision,
            PinnedRevision::Unfrozen("v1.2.3".to_owned())
        );
    }

    #[test]
    fn rejects_unsupported_action_reference_shapes() {
        let error = action_references(
            "steps:\n  - { uses: actions/checkout@v4 }\n",
            Path::new("workflow.yml"),
        )
        .expect_err("flow-style uses reference must not be skipped");

        assert!(error.to_string().contains("unsupported YAML shape"));
    }

    #[test]
    fn rejects_unsupported_pre_commit_repository_shapes() {
        let error = pre_commit_repositories(
            "repos:\n  - { repo: https://github.com/example/hook, rev: v1 }\n",
            Path::new(".pre-commit-config.yaml"),
        )
        .expect_err("flow-style repository must not be skipped");

        assert!(error.to_string().contains("unsupported YAML shape"));
    }
}
