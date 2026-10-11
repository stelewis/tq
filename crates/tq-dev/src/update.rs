//! Deterministic dependency and toolchain updates.

use std::collections::BTreeMap;
use std::path::Path;

use crate::action::{ActionPlan, PlannedAction};
use crate::error::DevError;
use crate::external_pins::{self, ExternalPinStatus, ExternalPinSurface};
use crate::invocation::Invocation;
use crate::manifest::{DevToolsManifest, NodeToolchain, ToolVersion};

pub struct ToolchainUpdates {
    pub rust: ToolVersion,
    pub python: ToolVersion,
    pub uv: ToolVersion,
    pub maturin: ToolVersion,
    pub node: ToolVersion,
    pub npm: ToolVersion,
}

pub fn discover(repo_root: &Path) -> Result<ToolchainUpdates, DevError> {
    let latest_node = release_version(repo_root, "https://github.com/nodejs/node.git")?;
    let invocation = Invocation::new(
        "curl",
        [
            "--fail",
            "--silent",
            "--show-error",
            "https://nodejs.org/dist/index.json",
        ],
    );
    let captured = invocation.capture(repo_root)?;
    if !captured.success {
        return Err(DevError::CommandFailed {
            program: invocation.program,
            args: invocation.args,
            code: captured.code,
        });
    }
    let npm = bundled_npm(&captured.output, &latest_node)?;
    Ok(ToolchainUpdates {
        rust: latest_stable_rust(repo_root)?,
        python: release_version(repo_root, "https://github.com/python/cpython.git")?,
        uv: release_version(repo_root, "https://github.com/astral-sh/uv.git")?,
        maturin: release_version(repo_root, "https://github.com/PyO3/maturin.git")?,
        node: latest_node,
        npm,
    })
}

fn release_version(repo_root: &Path, remote: &str) -> Result<ToolVersion, DevError> {
    let release = external_pins::latest_release(repo_root, remote)
        .map_err(|message| DevError::CommandOutputParse {
            program: "git ls-remote".to_owned(),
            message,
        })?
        .ok_or_else(|| DevError::CommandOutputParse {
            program: "git ls-remote".to_owned(),
            message: format!("no matching stable release for {remote}"),
        })?;
    ToolVersion::parse(release.tag.strip_prefix('v').unwrap_or(&release.tag)).map_err(|message| {
        DevError::CommandOutputParse {
            program: "git ls-remote".to_owned(),
            message,
        }
    })
}

fn bundled_npm(output: &str, node: &ToolVersion) -> Result<ToolVersion, DevError> {
    #[derive(serde::Deserialize)]
    struct NodeRelease {
        version: String,
        npm: Option<String>,
    }
    let releases: Vec<NodeRelease> =
        serde_json::from_str(output).map_err(|error| DevError::CommandOutputParse {
            program: "Node release index".to_owned(),
            message: error.to_string(),
        })?;
    let npm = releases
        .into_iter()
        .find(|release| release.version == format!("v{node}"))
        .and_then(|release| release.npm)
        .ok_or_else(|| DevError::CommandOutputParse {
            program: "Node release index".to_owned(),
            message: format!("missing bundled npm version for Node {node}"),
        })?;
    ToolVersion::parse(&npm).map_err(|message| DevError::CommandOutputParse {
        program: "Node release index".to_owned(),
        message,
    })
}

/// The dependency update plan: the single source of truth for what an update
/// applies.
pub fn plan(
    repo_root: &Path,
    latest: &ToolchainUpdates,
    report: external_pins::ExternalPinReport,
) -> Result<ActionPlan, DevError> {
    let manifest = DevToolsManifest::load(repo_root)?;
    let node = NodeToolchain::load(repo_root)?;
    let mut actions = plan_action_updates(repo_root, report)?;
    actions.extend(rust_updates(repo_root, &manifest.rust, &latest.rust));
    actions.extend(python_selector_update(repo_root, &latest.python)?);
    for (name, current, version) in [
        ("python", &manifest.python, &latest.python),
        ("uv", &manifest.uv, &latest.uv),
        ("maturin", &manifest.maturin, &latest.maturin),
    ] {
        if current == version {
            continue;
        }
        actions.push(PlannedAction::replace_text(
            &format!("Update {name} pin in dev tools manifest"),
            DevToolsManifest::path(repo_root),
            format!("{name} = \"{current}\""),
            format!("{name} = \"{version}\""),
        ));
        if name != "maturin" {
            actions.push(PlannedAction::replace_text(
                &format!("Update {name} setup default"),
                repo_root.join(".github/actions/setup-python-uv/action.yml"),
                format!("default: \"{current}\""),
                format!("default: \"{version}\""),
            ));
            actions.push(PlannedAction::replace_text(
                &format!("Update {name} publishing pin"),
                repo_root.join(".github/workflows/publish.yml"),
                format!(
                    "PUBLISH_{}_VERSION: \"{current}\"",
                    name.to_ascii_uppercase()
                ),
                format!(
                    "PUBLISH_{}_VERSION: \"{version}\"",
                    name.to_ascii_uppercase()
                ),
            ));
        }
    }
    if node.node != latest.node || node.npm != latest.npm {
        let path = repo_root.join("package.json");
        let original = crate::parse::read_to_string(&path)?;
        let mut document = crate::parse::read_json(&path)?;
        document["engines"]["node"] = serde_json::Value::String(latest.node.to_string());
        document["packageManager"] = serde_json::Value::String(format!("npm@{}", latest.npm));
        let updated =
            serde_json::to_string_pretty(&document).map_err(|error| DevError::InvalidInput {
                path: path.clone(),
                message: error.to_string(),
            })?;
        actions.push(PlannedAction::replace_text(
            "Update Node and bundled npm pins",
            path,
            original,
            format!("{updated}\n"),
        ));
    }
    actions.extend([
        PlannedAction::command(
            "Upgrade uv lockfile",
            Invocation::new("uv", ["lock", "--upgrade"]),
        ),
        PlannedAction::command(
            "Update npm dependencies",
            Invocation::new("npm", ["update"]),
        ),
        PlannedAction::command(
            "Synchronize npm lockfile toolchain metadata",
            Invocation::new(
                "npm",
                ["install", "--package-lock-only", "--ignore-scripts"],
            ),
        ),
        PlannedAction::command(
            "Freeze-update pre-commit hooks",
            Invocation::new("uv", ["run", "prek", "autoupdate", "--freeze"]),
        ),
    ]);
    Ok(ActionPlan::new("Dependency update", actions))
}

fn python_selector_update(
    repo_root: &Path,
    latest: &ToolVersion,
) -> Result<Vec<PlannedAction>, DevError> {
    let path = repo_root.join(".python-version");
    let original = crate::parse::read_to_string(&path)?;
    let updated = format!("{}\n", latest.minor_pin());
    if original == updated {
        return Ok(Vec::new());
    }
    Ok(vec![PlannedAction::replace_text(
        "Synchronize local Python selector",
        path,
        original,
        updated,
    )])
}

fn rust_updates(
    repo_root: &Path,
    current: &ToolVersion,
    latest: &ToolVersion,
) -> Vec<PlannedAction> {
    if current == latest {
        return Vec::new();
    }
    vec![
        PlannedAction::replace_text(
            "Update Rust pin in dev tools manifest",
            DevToolsManifest::path(repo_root),
            format!("rust = \"{current}\""),
            format!("rust = \"{latest}\""),
        ),
        PlannedAction::replace_text(
            "Update Rust toolchain file",
            repo_root.join("rust-toolchain.toml"),
            format!("channel = \"{current}\""),
            format!("channel = \"{latest}\""),
        ),
        PlannedAction::replace_text(
            "Update Cargo MSRV metadata",
            repo_root.join("Cargo.toml"),
            format!("rust-version = \"{}\"", current.minor_pin()),
            format!("rust-version = \"{}\"", latest.minor_pin()),
        ),
    ]
}

fn plan_action_updates(
    repo_root: &Path,
    report: external_pins::ExternalPinReport,
) -> Result<Vec<PlannedAction>, DevError> {
    let mut edits: BTreeMap<String, BTreeMap<usize, String>> = BTreeMap::new();
    for pin in report.pins {
        if pin.surface != ExternalPinSurface::GitHubAction {
            continue;
        }
        match pin.status {
            ExternalPinStatus::UpToDate => continue,
            ExternalPinStatus::InvalidPin | ExternalPinStatus::LookupFailed => {
                return Err(DevError::PolicyViolation {
                    details: format!(
                        "cannot update {}: {}",
                        pin.name,
                        pin.message.unwrap_or_default()
                    ),
                });
            }
            ExternalPinStatus::UpdateRequired => {}
        }
        let (Some(pinned), Some(sha), Some(tag)) = (pin.pinned, pin.latest_sha, pin.latest_tag)
        else {
            return Err(DevError::PolicyViolation {
                details: format!("incomplete release identity for {}", pin.name),
            });
        };
        if sha.len() != 40
            || !sha
                .bytes()
                .all(|byte| byte.is_ascii_digit() || (b'a'..=b'f').contains(&byte))
        {
            return Err(DevError::PolicyViolation {
                details: format!("invalid release commit for {}", pin.name),
            });
        }
        for source in pin.sources {
            let (file, line_number) =
                source
                    .rsplit_once(':')
                    .ok_or_else(|| DevError::PolicyViolation {
                        details: format!("invalid action source {source}"),
                    })?;
            let line_number: usize =
                line_number.parse().map_err(|_| DevError::PolicyViolation {
                    details: format!("invalid action source {source}"),
                })?;
            let contents = crate::parse::read_to_string(&repo_root.join(file))?;
            let line = contents
                .lines()
                .nth(line_number.saturating_sub(1))
                .ok_or_else(|| DevError::PolicyViolation {
                    details: format!("missing action source {source}"),
                })?;
            let reference = format!("{}@{pinned}", pin.name);
            if !line.contains(&reference) {
                return Err(DevError::PolicyViolation {
                    details: format!("action source changed at {source}"),
                });
            }
            let updated = line
                .split_once('#')
                .map_or(line, |(reference, _)| reference)
                .trim_end()
                .replace(&reference, &format!("{}@{sha}", pin.name));
            edits
                .entry(file.to_owned())
                .or_default()
                .insert(line_number, format!("{updated} # {tag}"));
        }
    }
    edits
        .into_iter()
        .map(|(file, replacements)| {
            let path = repo_root.join(&file);
            let original = crate::parse::read_to_string(&path)?;
            let updated: String = original
                .split_inclusive('\n')
                .enumerate()
                .map(|(index, line)| {
                    replacements.get(&(index + 1)).map_or_else(
                        || line.to_owned(),
                        |replacement| {
                            let body = line.trim_end_matches(['\r', '\n']);
                            format!("{replacement}{}", &line[body.len()..])
                        },
                    )
                })
                .collect();
            Ok(PlannedAction::replace_text(
                &format!("Update frozen Actions in {file}"),
                path,
                original,
                updated,
            ))
        })
        .collect()
}

pub fn latest_stable_rust(repo_root: &Path) -> Result<ToolVersion, DevError> {
    let invocation = Invocation::new("rustup", ["check"]);
    let captured = invocation.capture(repo_root)?;
    if !captured.success && captured.code != Some(100) {
        return Err(DevError::CommandFailed {
            program: invocation.program,
            args: invocation.args,
            code: captured.code,
        });
    }

    let version = captured
        .output
        .lines()
        .find_map(parse_stable_rustup_check_line)
        .ok_or_else(|| DevError::CommandOutputParse {
            program: "rustup check".to_owned(),
            message: "missing stable toolchain release in rustup check output".to_owned(),
        })?;
    ToolVersion::parse(&version).map_err(|message| DevError::CommandOutputParse {
        program: "rustup check".to_owned(),
        message,
    })
}

fn parse_stable_rustup_check_line(line: &str) -> Option<String> {
    let line = line.strip_prefix("stable-")?;
    let version_text = if let Some((_, updated)) = line.split_once("->") {
        updated
    } else {
        let (_, current) = line.split_once("up to date:")?;
        current
    };
    version_text
        .split_whitespace()
        .next()
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::{bundled_npm, parse_stable_rustup_check_line, plan_action_updates};
    use crate::action::ActionPlan;
    use crate::external_pins::{
        ExternalPin, ExternalPinReport, ExternalPinStatus, ExternalPinSurface,
    };
    use crate::manifest::ToolVersion;

    #[test]
    fn bundled_npm_uses_selected_node_release() {
        let index =
            r#"[{"version":"v26.5.0","npm":"11.18.0"},{"version":"v26.4.0","npm":"11.17.0"}]"#;
        let node = ToolVersion::parse("26.4.0").expect("node");
        assert_eq!(
            bundled_npm(index, &node).expect("bundled npm").as_str(),
            "11.17.0"
        );
        assert!(bundled_npm("[]", &node).is_err());
        assert!(bundled_npm(r#"[{"version":"v26.4.0"}]"#, &node).is_err());
        assert!(bundled_npm("not JSON", &node).is_err());
    }

    #[test]
    fn action_plan_updates_frozen_reference_and_release_comment() {
        let temp = tempfile::tempdir().expect("tempdir");
        let old = "a".repeat(40);
        let new = "b".repeat(40);
        let line = format!("    uses: actions/checkout@{old} # v7.0.0");
        std::fs::write(
            temp.path().join("ci.yml"),
            format!("steps:\n{line}\n{line}\n"),
        )
        .expect("workflow with repeated references");
        let report = ExternalPinReport {
            drift_detected: true,
            pins: vec![ExternalPin {
                surface: ExternalPinSurface::GitHubAction,
                name: "actions/checkout".to_owned(),
                pinned: Some(old),
                latest_tag: Some("v7.0.1".to_owned()),
                latest_sha: Some(new.clone()),
                status: ExternalPinStatus::UpdateRequired,
                message: None,
                sources: vec![
                    "ci.yml:2".to_owned(),
                    "ci.yml:2".to_owned(),
                    "ci.yml:3".to_owned(),
                ],
            }],
        };
        let actions = plan_action_updates(temp.path(), report).expect("action plan");
        ActionPlan::new("Action updates", actions)
            .apply(temp.path())
            .expect("apply deduplicated references");
        assert_eq!(
            std::fs::read_to_string(temp.path().join("ci.yml")).expect("updated workflow"),
            format!(
                "steps:\n    uses: actions/checkout@{new} # v7.0.1\n    uses: actions/checkout@{new} # v7.0.1\n"
            )
        );
    }

    #[test]
    fn parses_rustup_stable_release_when_current() {
        assert_eq!(
            parse_stable_rustup_check_line(
                "stable-x86_64-apple-darwin - up to date: 1.97.0 (2d8144b78 2026-07-07)",
            ),
            Some("1.97.0".to_owned())
        );
    }

    #[test]
    fn parses_rustup_stable_release_when_update_is_available() {
        assert_eq!(
            parse_stable_rustup_check_line(
                "stable-x86_64-apple-darwin - Update available : \
                 1.96.1 (abc 2026-06-01) -> 1.97.0 (2d8144b78 2026-07-07)",
            ),
            Some("1.97.0".to_owned())
        );
    }
}
