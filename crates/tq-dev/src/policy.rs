//! Repository policy verification: pinned tool surfaces must agree with the
//! dev-tools manifest, and the release policy aggregates all repository
//! invariants.

use std::path::Path;

use crate::error::DevError;
use crate::manifest::DevToolsManifest;
use crate::parse;
use crate::{change_scope, dependabot, external_pins, release, workflow_policy, workspace_version};

/// Verifies every repository surface that pins a developer tool version
/// against `.github/dev-tools.toml`.
pub fn verify_tool_pins(repo_root: &Path) -> Result<(), DevError> {
    let manifest = DevToolsManifest::load(repo_root)?;
    let mut violations = Vec::new();

    verify_rust_toolchain(repo_root, &manifest, &mut violations)?;
    verify_cargo_msrv(repo_root, &manifest, &mut violations)?;
    verify_mise_tools(repo_root, &manifest, &mut violations)?;
    verify_node_package(repo_root, &manifest, &mut violations)?;
    verify_mise_action(repo_root, &manifest, &mut violations)?;
    verify_python_uv_action(repo_root, &manifest, &mut violations)?;
    verify_rust_maintenance_action(repo_root, &manifest, &mut violations)?;
    verify_maturin_release_builder(repo_root, &mut violations)?;

    if violations.is_empty() {
        return Ok(());
    }

    Err(DevError::PolicyViolation {
        details: violations.join("\n"),
    })
}

/// The release-policy gate: workspace version consistency, Dependabot
/// coverage, and pinned tool surfaces.
pub fn verify_release_policy(repo_root: &Path) -> Result<(), DevError> {
    workspace_version::verify_workspace_version(repo_root)?;
    dependabot::verify_dependabot(repo_root)?;
    verify_tool_pins(repo_root)
}

pub fn verify_automation_policy(repo_root: &Path) -> Result<(), DevError> {
    change_scope::verify_tracked_path_coverage(repo_root)?;
    workflow_policy::verify_workflow_hardening(repo_root)?;
    external_pins::verify_action_pins(repo_root)?;
    external_pins::verify_pre_commit_pins(repo_root)?;
    dependabot::verify_dependabot(repo_root)
}

fn verify_rust_toolchain(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    violations: &mut Vec<String>,
) -> Result<(), DevError> {
    let path = repo_root.join("rust-toolchain.toml");
    let document = parse::read_toml(&path)?;
    require_equal(
        violations,
        "rust-toolchain.toml toolchain.channel",
        manifest.rust.as_str(),
        &parse::required_string(&document, &["toolchain", "channel"], &path)?,
    );
    let components = parse::required_string_array(&document, &["toolchain", "components"], &path)?;
    for component in ["rustfmt", "clippy"] {
        require_array_contains(
            violations,
            "rust-toolchain.toml toolchain.components",
            &components,
            component,
        );
    }
    Ok(())
}

fn verify_cargo_msrv(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    violations: &mut Vec<String>,
) -> Result<(), DevError> {
    let path = repo_root.join("Cargo.toml");
    let document = parse::read_toml(&path)?;
    require_equal(
        violations,
        "Cargo.toml workspace.package.rust-version",
        &manifest.rust.minor_pin(),
        &parse::required_string(&document, &["workspace", "package", "rust-version"], &path)?,
    );
    Ok(())
}

fn verify_mise_tools(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    violations: &mut Vec<String>,
) -> Result<(), DevError> {
    let path = repo_root.join("mise.toml");
    let document = parse::read_toml(&path)?;
    require_equal(
        violations,
        "mise.toml tools.node",
        manifest.node.as_str(),
        &parse::required_string(&document, &["tools", "node"], &path)?,
    );
    require_equal(
        violations,
        "mise.toml tools.actionlint",
        manifest.actionlint.as_str(),
        &parse::required_string(&document, &["tools", "actionlint"], &path)?,
    );
    require_equal(
        violations,
        "mise.toml tools.shellcheck",
        manifest.shellcheck.as_str(),
        &parse::required_string(&document, &["tools", "shellcheck"], &path)?,
    );

    let lock_path = repo_root.join("mise.lock");
    let lock = parse::read_toml(&lock_path)?;
    verify_mise_lock_tool(
        &lock,
        &lock_path,
        "node",
        "core:node",
        manifest.node.as_str(),
        violations,
    )?;
    verify_mise_lock_tool(
        &lock,
        &lock_path,
        "actionlint",
        "aqua:rhysd/actionlint",
        manifest.actionlint.as_str(),
        violations,
    )?;
    verify_mise_lock_tool(
        &lock,
        &lock_path,
        "shellcheck",
        "aqua:koalaman/shellcheck",
        manifest.shellcheck.as_str(),
        violations,
    )?;
    Ok(())
}

fn verify_mise_lock_tool(
    document: &toml::Value,
    source: &Path,
    tool: &str,
    expected_backend: &str,
    expected_version: &str,
    violations: &mut Vec<String>,
) -> Result<(), DevError> {
    const PLATFORMS: &[&str] = &["linux-x64", "macos-arm64", "macos-x64", "windows-x64"];

    let entries = document
        .get("tools")
        .and_then(toml::Value::as_table)
        .and_then(|tools| tools.get(tool))
        .and_then(toml::Value::as_array)
        .ok_or_else(|| DevError::InvalidInput {
            path: source.to_path_buf(),
            message: format!("missing tools.{tool} lock entry"),
        })?;
    if entries.len() != 1 {
        return Err(DevError::InvalidInput {
            path: source.to_path_buf(),
            message: format!("tools.{tool} must contain exactly one lock entry"),
        });
    }
    let entry = entries[0]
        .as_table()
        .ok_or_else(|| DevError::InvalidInput {
            path: source.to_path_buf(),
            message: format!("tools.{tool} lock entry must be a table"),
        })?;
    let version = entry
        .get("version")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| DevError::InvalidInput {
            path: source.to_path_buf(),
            message: format!("tools.{tool} lock entry is missing version"),
        })?;
    let backend = entry
        .get("backend")
        .and_then(toml::Value::as_str)
        .ok_or_else(|| DevError::InvalidInput {
            path: source.to_path_buf(),
            message: format!("tools.{tool} lock entry is missing backend"),
        })?;
    require_equal(
        violations,
        &format!("mise.lock tools.{tool}.version"),
        expected_version,
        version,
    );
    require_equal(
        violations,
        &format!("mise.lock tools.{tool}.backend"),
        expected_backend,
        backend,
    );

    for platform in PLATFORMS {
        let platform_key = format!("platforms.{platform}");
        let locked = entry
            .get(&platform_key)
            .and_then(toml::Value::as_table)
            .ok_or_else(|| DevError::InvalidInput {
                path: source.to_path_buf(),
                message: format!("tools.{tool} is missing locked platform {platform}"),
            })?;
        let checksum = locked
            .get("checksum")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| DevError::InvalidInput {
                path: source.to_path_buf(),
                message: format!("tools.{tool}.{platform} is missing checksum"),
            })?;
        let url = locked
            .get("url")
            .and_then(toml::Value::as_str)
            .ok_or_else(|| DevError::InvalidInput {
                path: source.to_path_buf(),
                message: format!("tools.{tool}.{platform} is missing URL"),
            })?;
        if !checksum.starts_with("sha256:") || url.is_empty() {
            violations.push(format!(
                "mise.lock tools.{tool}.{platform} must contain a SHA-256 checksum and URL"
            ));
        }
    }
    Ok(())
}

fn verify_node_package(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    violations: &mut Vec<String>,
) -> Result<(), DevError> {
    let path = repo_root.join("package.json");
    let document = parse::read_json(&path)?;
    require_equal(
        violations,
        "package.json packageManager",
        &format!("npm@{}", manifest.npm),
        &parse::required_json_string(&document, &["packageManager"], &path)?,
    );
    require_equal(
        violations,
        "package.json engines.node",
        manifest.node.as_str(),
        &parse::required_json_string(&document, &["engines", "node"], &path)?,
    );
    require_equal(
        violations,
        "package.json engines.npm",
        manifest.npm.as_str(),
        &parse::required_json_string(&document, &["engines", "npm"], &path)?,
    );
    Ok(())
}

fn verify_mise_action(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    violations: &mut Vec<String>,
) -> Result<(), DevError> {
    let path = repo_root.join(".github/actions/setup-mise/action.yml");
    let contents = parse::read_to_string(&path)?;
    let inputs = parse::action_inputs(&contents, &path)?;
    require_equal(
        violations,
        ".github/actions/setup-mise/action.yml mise-version default",
        manifest.mise.as_str(),
        &parse::required_action_input_default(&inputs, "mise-version", &path)?,
    );
    require_equal(
        violations,
        ".github/actions/setup-mise/action.yml mise-action version input",
        "${{ inputs.mise-version }}",
        &parse::required_action_step_with_value(&contents, "jdx/mise-action@", "version", &path)?,
    );
    Ok(())
}

fn verify_python_uv_action(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    violations: &mut Vec<String>,
) -> Result<(), DevError> {
    let path = repo_root.join(".github/actions/setup-python-uv/action.yml");
    let contents = parse::read_to_string(&path)?;
    let inputs = parse::action_inputs(&contents, &path)?;
    require_equal(
        violations,
        ".github/actions/setup-python-uv/action.yml python-version default",
        manifest.python.as_str(),
        &parse::required_action_input_default(&inputs, "python-version", &path)?,
    );
    require_equal(
        violations,
        ".github/actions/setup-python-uv/action.yml uv-version default",
        manifest.uv.as_str(),
        &parse::required_action_input_default(&inputs, "uv-version", &path)?,
    );
    require_equal(
        violations,
        ".github/actions/setup-python-uv/action.yml setup-uv version input",
        "${{ inputs.uv-version }}",
        &parse::required_action_step_with_value(
            &contents,
            "astral-sh/setup-uv@",
            "version",
            &path,
        )?,
    );
    Ok(())
}

fn verify_rust_maintenance_action(
    repo_root: &Path,
    manifest: &DevToolsManifest,
    violations: &mut Vec<String>,
) -> Result<(), DevError> {
    let path = repo_root.join(".github/actions/setup-rust-maintenance-tools/action.yml");
    let contents = parse::read_to_string(&path)?;
    let inputs = parse::action_inputs(&contents, &path)?;
    for (input, pinned) in [
        ("cargo-outdated-version", &manifest.cargo_outdated),
        ("cargo-audit-version", &manifest.cargo_audit),
        ("cargo-deny-version", &manifest.cargo_deny),
    ] {
        require_equal(
            violations,
            &format!(".github/actions/setup-rust-maintenance-tools/action.yml {input}"),
            pinned.as_str(),
            &parse::required_action_input_default(&inputs, input, &path)?,
        );
    }
    Ok(())
}

fn verify_maturin_release_builder(
    repo_root: &Path,
    violations: &mut Vec<String>,
) -> Result<(), DevError> {
    let tools = release::build_tool_requirements(repo_root)?;

    let release_plan = release::plan(repo_root)?;
    let release_commands = release_plan
        .actions
        .iter()
        .map(|action| action.action.detail())
        .collect::<Vec<_>>()
        .join("\n");
    require_contains(
        violations,
        "cargo dev release build plan",
        &release_commands,
        &format!("--with {} -- maturin build", tools.maturin),
    );

    let ci_path = repo_root.join(".github/workflows/ci.yml");
    let ci = parse::read_to_string(&ci_path)?;
    for forbidden in ["maturin>=", "maturin[zig]>=", "maturin==", "maturin[zig]=="] {
        require_not_contains(violations, ".github/workflows/ci.yml", &ci, forbidden);
    }
    require_contains(
        violations,
        ".github/workflows/ci.yml",
        &ci,
        "cargo dev release build-tool-requirements --repo-root .",
    );
    require_contains(
        violations,
        ".github/workflows/ci.yml",
        &ci,
        "maturin_requirement='${{ steps.release-tools.outputs.maturin }}'",
    );
    require_contains(
        violations,
        ".github/workflows/ci.yml",
        &ci,
        "maturin_requirement='${{ steps.release-tools.outputs.maturin_zig }}'",
    );
    require_contains(
        violations,
        ".github/workflows/ci.yml",
        &ci,
        "--with \"$maturin_requirement\" --",
    );
    require_contains(
        violations,
        ".github/workflows/ci.yml",
        &ci,
        "--with \"${{ steps.release-tools.outputs.maturin_zig }}\" --",
    );

    Ok(())
}

fn require_equal(violations: &mut Vec<String>, field: &str, expected: &str, actual: &str) {
    if actual != expected {
        violations.push(format!("{field} must be {expected:?}, found {actual:?}"));
    }
}

fn require_array_contains(
    violations: &mut Vec<String>,
    field: &str,
    values: &[String],
    expected: &str,
) {
    if !values.iter().any(|value| value == expected) {
        violations.push(format!("{field} must contain {expected:?}"));
    }
}

fn require_contains(violations: &mut Vec<String>, field: &str, contents: &str, expected: &str) {
    if !contents.contains(expected) {
        violations.push(format!("{field} must contain {expected:?}"));
    }
}

fn require_not_contains(
    violations: &mut Vec<String>,
    field: &str,
    contents: &str,
    forbidden: &str,
) {
    if contents.contains(forbidden) {
        violations.push(format!("{field} must not contain {forbidden:?}"));
    }
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::verify_mise_lock_tool;

    #[test]
    fn mise_lock_verification_rejects_missing_platforms_and_backend_drift() {
        let document = concat!(
            "[[tools.actionlint]]\n",
            "version = \"1.7.12\"\n",
            "backend = \"aqua:unexpected/actionlint\"\n",
            "[tools.actionlint.\"platforms.linux-x64\"]\n",
            "checksum = \"sha256:fixture\"\n",
            "url = \"https://example.invalid/actionlint\"\n",
        )
        .parse::<toml::Table>()
        .map(toml::Value::Table)
        .expect("fixture lock must parse");
        let mut violations = Vec::new();

        let error = verify_mise_lock_tool(
            &document,
            Path::new("mise.lock"),
            "actionlint",
            "aqua:rhysd/actionlint",
            "1.7.12",
            &mut violations,
        )
        .expect_err("missing platforms must fail");

        assert!(error.to_string().contains("macos-arm64"));
        assert!(
            violations
                .iter()
                .any(|violation| violation.contains("backend"))
        );
    }
}
