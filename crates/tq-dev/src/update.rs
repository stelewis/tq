//! Deterministic dependency and toolchain updates.

use std::path::Path;

use crate::action::{ActionPlan, PlannedAction};
use crate::error::DevError;
use crate::invocation::Invocation;
use crate::manifest::{DevToolsManifest, ToolVersion};

/// The dependency update plan: the single source of truth for what an update
/// applies.
pub fn plan(repo_root: &Path) -> Result<ActionPlan, DevError> {
    let manifest = DevToolsManifest::load(repo_root)?;
    let latest_rust = latest_stable_rust(repo_root)?;

    let mut actions = Vec::new();
    if latest_rust != manifest.rust {
        actions.extend([
            PlannedAction::replace_text(
                "Update Rust pin in dev tools manifest",
                DevToolsManifest::path(repo_root),
                format!("rust = \"{}\"", manifest.rust),
                format!("rust = \"{latest_rust}\""),
            ),
            PlannedAction::replace_text(
                "Update Rust toolchain file",
                repo_root.join("rust-toolchain.toml"),
                format!("channel = \"{}\"", manifest.rust),
                format!("channel = \"{latest_rust}\""),
            ),
            PlannedAction::replace_text(
                "Update Cargo MSRV metadata",
                repo_root.join("Cargo.toml"),
                format!("rust-version = \"{}\"", manifest.rust.minor_pin()),
                format!("rust-version = \"{}\"", latest_rust.minor_pin()),
            ),
        ]);
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
            "Freeze-update pre-commit hooks",
            Invocation::new("uv", ["run", "prek", "autoupdate", "--freeze"]),
        ),
    ]);
    Ok(ActionPlan::new("Dependency update", actions))
}

pub fn latest_stable_rust(repo_root: &Path) -> Result<ToolVersion, DevError> {
    let invocation = Invocation::new("rustup", ["check"]);
    let captured = invocation.capture(repo_root)?;
    if !captured.success {
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
    use super::parse_stable_rustup_check_line;

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
