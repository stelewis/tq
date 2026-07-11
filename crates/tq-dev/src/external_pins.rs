//! Drift audit for frozen external pins: GitHub Action refs and pre-commit
//! hook revisions.

use std::collections::BTreeMap;
use std::path::Path;
use std::process::Command;

use serde::Serialize;

use crate::error::DevError;
use crate::label::labeled_enum;
use crate::parse;

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct ExternalPinReport {
    pub title: String,
    pub drift_detected: bool,
    pub results: Vec<ExternalPinResult>,
}

#[derive(Debug, Eq, PartialEq, Serialize)]
pub struct ExternalPinResult {
    pub surface: ExternalPinSurface,
    pub source: String,
    pub name: String,
    pub remote: Option<String>,
    pub pinned: Option<String>,
    pub latest: Option<String>,
    pub status: ExternalPinStatus,
    pub message: Option<String>,
}

labeled_enum! {
    #[derive(Ord, PartialOrd)]
    pub enum ExternalPinSurface {
        GitHubAction => "GitHub Action",
        PreCommitHook => "pre-commit hook",
    }
}

labeled_enum! {
    pub enum ExternalPinStatus {
        UpToDate => "up to date",
        UpdateRequired => "update required",
        InvalidPin => "invalid pin",
        LookupFailed => "lookup failed",
    }
}

#[derive(Debug, Eq, PartialEq)]
struct ExternalPinRef {
    surface: ExternalPinSurface,
    source: String,
    name: String,
    remote: Option<String>,
    pinned: Option<String>,
    lookup_message: Option<String>,
}

pub fn audit_external_pin_drift(repo_root: &Path) -> Result<ExternalPinReport, DevError> {
    let mut refs = Vec::new();
    refs.extend(action_refs(repo_root)?);
    refs.extend(pre_commit_refs(repo_root)?);
    refs.sort_by(|left, right| {
        (&left.surface, &left.source, &left.name).cmp(&(&right.surface, &right.source, &right.name))
    });

    let mut cache = BTreeMap::new();
    let results = refs
        .iter()
        .map(|pin| resolve_pin(repo_root, pin, &mut cache))
        .collect::<Vec<_>>();
    let drift_detected = results
        .iter()
        .any(|result| result.status != ExternalPinStatus::UpToDate);

    Ok(ExternalPinReport {
        title: "Frozen external pin review".to_owned(),
        drift_detected,
        results,
    })
}

fn action_refs(repo_root: &Path) -> Result<Vec<ExternalPinRef>, DevError> {
    let files = git_ls_files(
        repo_root,
        &[
            ".github/workflows/*.yml",
            ".github/workflows/*.yaml",
            ".github/actions/**/action.yml",
            ".github/actions/**/action.yaml",
        ],
    )?;
    let mut refs = Vec::new();

    for file in files {
        let path = repo_root.join(&file);
        let contents = parse::read_to_string(&path)?;
        for line in contents.lines() {
            let Some(reference) = parse_action_uses(line) else {
                continue;
            };
            if reference.starts_with("./") || reference.starts_with("docker://") {
                continue;
            }

            let (name, pinned) = split_pin(&reference);
            let remote = github_action_remote(&name);
            refs.push(ExternalPinRef {
                surface: ExternalPinSurface::GitHubAction,
                source: file.clone(),
                name: name.clone(),
                remote: remote.clone(),
                pinned,
                lookup_message: remote.is_none().then(|| {
                    "unsupported GitHub Action reference for automated drift lookup".to_owned()
                }),
            });
        }
    }

    Ok(refs)
}

fn pre_commit_refs(repo_root: &Path) -> Result<Vec<ExternalPinRef>, DevError> {
    let source = ".pre-commit-config.yaml";
    let path = repo_root.join(source);
    if !path.exists() {
        return Ok(Vec::new());
    }
    let contents = parse::read_to_string(&path)?;
    let mut refs = Vec::new();
    let mut current_repo: Option<String> = None;

    for line in contents.lines() {
        let trimmed = line.trim();
        if let Some(repo) = trimmed.strip_prefix("- repo:") {
            current_repo = Some(parse::yaml_scalar(repo).to_owned());
            continue;
        }

        let Some(rev) = trimmed.strip_prefix("rev:") else {
            continue;
        };
        let Some(repo) = current_repo.take() else {
            continue;
        };
        if repo == "local" {
            continue;
        }

        let remote = github_remote_from_pre_commit(&repo);
        refs.push(ExternalPinRef {
            surface: ExternalPinSurface::PreCommitHook,
            source: source.to_owned(),
            name: repo.clone(),
            remote: remote.clone(),
            pinned: Some(parse::yaml_scalar(rev).to_owned()),
            lookup_message: remote.is_none().then(|| {
                "unsupported pre-commit repository URL for automated drift lookup".to_owned()
            }),
        });
    }

    Ok(refs)
}

fn resolve_pin(
    repo_root: &Path,
    pin: &ExternalPinRef,
    cache: &mut BTreeMap<String, Result<Option<ReleaseRef>, String>>,
) -> ExternalPinResult {
    let Some(pinned) = pin.pinned.as_deref() else {
        return result(
            pin,
            None,
            ExternalPinStatus::InvalidPin,
            Some("missing @ref".to_owned()),
        );
    };
    if !is_full_sha(pinned) {
        return result(
            pin,
            None,
            ExternalPinStatus::InvalidPin,
            Some("pinned ref is not a full 40-character commit SHA".to_owned()),
        );
    }

    let Some(remote) = pin.remote.as_deref() else {
        return result(
            pin,
            None,
            ExternalPinStatus::LookupFailed,
            pin.lookup_message.clone(),
        );
    };

    let release = cache
        .entry(remote.to_owned())
        .or_insert_with(|| latest_release(repo_root, remote))
        .clone();

    match release {
        Ok(Some(release)) => {
            let latest = format!("{} ({})", release.tag, release.sha);
            let status = if release.sha == pinned {
                ExternalPinStatus::UpToDate
            } else {
                ExternalPinStatus::UpdateRequired
            };
            result(pin, Some(latest), status, None)
        }
        Ok(None) => result(
            pin,
            None,
            ExternalPinStatus::LookupFailed,
            Some(format!("no SemVer release tags were found for {remote}")),
        ),
        Err(message) => result(pin, None, ExternalPinStatus::LookupFailed, Some(message)),
    }
}

fn result(
    pin: &ExternalPinRef,
    latest: Option<String>,
    status: ExternalPinStatus,
    message: Option<String>,
) -> ExternalPinResult {
    ExternalPinResult {
        surface: pin.surface,
        source: pin.source.clone(),
        name: pin.name.clone(),
        remote: pin.remote.clone(),
        pinned: pin.pinned.clone(),
        latest: latest.or_else(|| message.clone()),
        status,
        message,
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct ReleaseRef {
    tag: String,
    sha: String,
}

fn latest_release(repo_root: &Path, remote: &str) -> Result<Option<ReleaseRef>, String> {
    let output = Command::new("git")
        .args(["ls-remote", "--tags", remote])
        .current_dir(repo_root)
        .output()
        .map_err(|source| source.to_string())?;
    if !output.status.success() {
        return Err(String::from_utf8_lossy(&output.stderr).trim().to_owned());
    }

    let mut tags: BTreeMap<String, TagEntry> = BTreeMap::new();
    for line in String::from_utf8_lossy(&output.stdout).lines() {
        let Some((sha, git_ref)) = line.split_once(char::is_whitespace) else {
            continue;
        };
        let Some(tag) = git_ref.trim().strip_prefix("refs/tags/") else {
            continue;
        };
        let (tag, peeled) = tag
            .strip_suffix("^{}")
            .map_or((tag, false), |stripped| (stripped, true));
        let Some(version) = semver_key(tag) else {
            continue;
        };

        let entry = tags.entry(tag.to_owned()).or_insert_with(|| TagEntry {
            version,
            sha: sha.to_owned(),
            peeled_sha: None,
        });
        if peeled {
            entry.peeled_sha = Some(sha.to_owned());
        } else {
            sha.clone_into(&mut entry.sha);
        }
    }

    Ok(tags
        .into_iter()
        .max_by_key(|(_, entry)| entry.version)
        .map(|(tag, entry)| ReleaseRef {
            tag,
            sha: entry.peeled_sha.unwrap_or(entry.sha),
        }))
}

#[derive(Debug, Eq, PartialEq)]
struct TagEntry {
    version: (u64, u64, u64),
    sha: String,
    peeled_sha: Option<String>,
}

fn semver_key(tag: &str) -> Option<(u64, u64, u64)> {
    let tag = tag.strip_prefix('v').unwrap_or(tag);
    let mut parts = tag.split('.');
    let major = parts.next()?.parse().ok()?;
    let minor = parts.next()?.parse().ok()?;
    let patch = parts.next()?.parse().ok()?;
    if parts.next().is_some() {
        return None;
    }
    Some((major, minor, patch))
}

fn parse_action_uses(line: &str) -> Option<String> {
    let trimmed = line
        .trim_start()
        .strip_prefix('-')
        .unwrap_or(line)
        .trim_start();
    let value = trimmed.strip_prefix("uses:")?.trim();
    Some(parse::yaml_scalar(value).to_owned())
}

fn split_pin(reference: &str) -> (String, Option<String>) {
    reference.split_once('@').map_or_else(
        || (reference.to_owned(), None),
        |(name, pinned)| (name.to_owned(), Some(pinned.to_owned())),
    )
}

fn github_remote_from_pre_commit(repo: &str) -> Option<String> {
    let path = repo
        .strip_prefix("https://github.com/")
        .map(|path| path.trim_end_matches('/').trim_end_matches(".git"))
        .or_else(|| {
            repo.strip_prefix("git@github.com:")
                .map(|path| path.trim_end_matches(".git"))
        })?;

    let mut parts = path.split('/');
    let owner = parts.next()?;
    let name = parts.next()?;
    if parts.next().is_some() || owner.is_empty() || name.is_empty() {
        return None;
    }
    Some(format!("https://github.com/{owner}/{name}.git"))
}

fn github_action_remote(name: &str) -> Option<String> {
    let mut parts = name.split('/');
    let owner = parts.next()?;
    let repo = parts.next()?;
    if owner.is_empty() || repo.is_empty() {
        return None;
    }
    Some(format!("https://github.com/{owner}/{repo}.git"))
}

fn is_full_sha(value: &str) -> bool {
    value.len() == 40
        && value
            .bytes()
            .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
}

fn git_ls_files(repo_root: &Path, patterns: &[&str]) -> Result<Vec<String>, DevError> {
    let output = Command::new("git")
        .arg("ls-files")
        .args(patterns)
        .current_dir(repo_root)
        .output()
        .map_err(|source| DevError::GitIo {
            args: format!("ls-files {}", patterns.join(" ")),
            source,
        })?;
    if !output.status.success() {
        return Err(DevError::Git {
            args: format!("ls-files {}", patterns.join(" ")),
            stderr: String::from_utf8_lossy(&output.stderr).trim().to_owned(),
        });
    }

    Ok(String::from_utf8_lossy(&output.stdout)
        .lines()
        .map(ToOwned::to_owned)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::{
        github_action_remote, github_remote_from_pre_commit, is_full_sha, parse_action_uses,
        semver_key,
    };

    #[test]
    fn parses_action_uses_with_quotes_and_comment() {
        assert_eq!(
            parse_action_uses(
                "  - uses: 'actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0' # frozen"
            ),
            Some("actions/checkout@9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0".to_owned())
        );
    }

    #[test]
    fn normalizes_supported_pre_commit_github_remotes() {
        assert_eq!(
            github_remote_from_pre_commit("https://github.com/astral-sh/ruff-pre-commit.git"),
            Some("https://github.com/astral-sh/ruff-pre-commit.git".to_owned())
        );
        assert_eq!(
            github_remote_from_pre_commit("git@github.com:astral-sh/ruff-pre-commit.git"),
            Some("https://github.com/astral-sh/ruff-pre-commit.git".to_owned())
        );
    }

    #[test]
    fn resolves_action_subpaths_to_owner_repo_remote() {
        assert_eq!(
            github_action_remote("actions/cache/restore"),
            Some("https://github.com/actions/cache.git".to_owned())
        );
    }

    #[test]
    fn accepts_only_three_part_semver_release_tags() {
        assert_eq!(semver_key("v1.2.3"), Some((1, 2, 3)));
        assert_eq!(semver_key("1.2.3"), Some((1, 2, 3)));
        assert_eq!(semver_key("v1.2"), None);
        assert_eq!(semver_key("v1.2.3-beta.1"), None);
    }

    #[test]
    fn accepts_only_lowercase_full_commit_sha_pins() {
        assert!(is_full_sha("9c091bb21b7c1c1d1991bb908d89e4e9dddfe3e0")); // pragma: allowlist secret
        assert!(!is_full_sha("9C091BB21B7C1C1D1991BB908D89E4E9DDDFE3E0")); // pragma: allowlist secret
        assert!(!is_full_sha("v1.2.3"));
    }
}
