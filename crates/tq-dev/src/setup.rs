//! Developer environment setup from the pinned tool manifest.

use std::path::{Path, PathBuf};

use crate::action::{ActionPlan, PlannedAction};
use crate::error::DevError;
use crate::invocation;
use crate::invocation::Invocation;
use crate::manifest::{DevToolsManifest, ToolVersion};

/// The developer setup plan: the single source of truth for what setup
/// installs. Callers verify native build prerequisites before applying.
pub fn plan(repo_root: &Path) -> Result<ActionPlan, DevError> {
    let manifest = DevToolsManifest::load(repo_root)?;
    let mut actions = vec![
        PlannedAction::command(
            "Install pinned Rust toolchain",
            Invocation::new(
                "rustup",
                [
                    "toolchain",
                    "install",
                    manifest.rust.as_str(),
                    "--profile",
                    "minimal",
                    "--component",
                    "rustfmt",
                    "--component",
                    "clippy",
                ],
            ),
        ),
        PlannedAction::command(
            "Install pinned mise tools",
            Invocation::new("mise", ["install"]),
        ),
        PlannedAction::command(
            "Install pinned Python",
            Invocation::new("uv", ["python", "install", manifest.python.as_str()]),
        ),
        PlannedAction::command(
            "Sync locked Python dependencies",
            Invocation::new("uv", ["sync", "--locked"]),
        ),
        PlannedAction::command(
            "Install locked npm dependencies",
            Invocation::new("npm", ["ci"]),
        ),
        PlannedAction::command(
            "Install pre-commit hooks",
            Invocation::new("uv", ["run", "prek", "install", "--install-hooks"]),
        ),
    ];
    actions.extend(cargo_tool_install_actions(repo_root, &manifest));
    Ok(ActionPlan::new("Developer setup", actions))
}

/// Removal of the local Cargo maintenance-tool build cache.
#[must_use]
pub fn cleanup_plan(repo_root: &Path) -> ActionPlan {
    ActionPlan::new(
        "Developer environment cleanup",
        vec![PlannedAction::remove_path(
            "Remove Cargo maintenance-tool build cache",
            repo_root.join(CARGO_TOOLS_TARGET_DIR),
        )],
    )
}

const CARGO_TOOLS_TARGET_DIR: &str = "target/cargo-tools";

fn cargo_tool_install_actions(repo_root: &Path, manifest: &DevToolsManifest) -> Vec<PlannedAction> {
    let tools: [(&str, &str, &ToolVersion, &[&str]); 3] = [
        (
            "cargo-outdated",
            "outdated",
            &manifest.cargo_outdated,
            &["cargo-outdated"],
        ),
        ("cargo-deny", "deny", &manifest.cargo_deny, &["cargo-deny"]),
        (
            "cargo-audit",
            "audit",
            &manifest.cargo_audit,
            &["cargo-audit", "cargo-audit-audit"],
        ),
    ];

    let mut actions = Vec::new();
    for (crate_name, subcommand, pinned, stale_binaries) in tools {
        if cargo_subcommand_matches(subcommand, pinned) {
            continue;
        }
        if let Some(bin_dir) = cargo_bin_dir() {
            for binary in stale_binaries {
                actions.push(PlannedAction::remove_path(
                    "Remove stale Cargo tool binary",
                    bin_dir.join(binary),
                ));
            }
        }
        actions.push(PlannedAction::command(
            "Install pinned Cargo maintenance tool",
            Invocation::new(
                "cargo",
                [
                    "install",
                    "--force",
                    "--locked",
                    &format!("{crate_name}@{pinned}"),
                ],
            )
            .with_env(vec![(
                "CARGO_TARGET_DIR".to_owned(),
                repo_root.join(CARGO_TOOLS_TARGET_DIR).display().to_string(),
            )]),
        ));
    }

    actions
}

fn cargo_subcommand_matches(subcommand: &str, expected: &ToolVersion) -> bool {
    invocation::probe("cargo", &[subcommand, "--version"])
        .is_ok_and(|captured| captured.success && expected.matches_version_output(&captured.output))
}

fn cargo_bin_dir() -> Option<PathBuf> {
    if let Some(cargo_home) = std::env::var_os("CARGO_HOME") {
        return Some(Path::new(&cargo_home).join("bin"));
    }

    std::env::var_os("HOME").map(|home| Path::new(&home).join(".cargo/bin"))
}
