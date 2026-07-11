//! Repository policy verification: pinned tool surfaces must agree with the
//! dev-tools manifest, and the release policy aggregates all repository
//! invariants.

use std::path::Path;

use crate::error::DevError;
use crate::manifest::DevToolsManifest;
use crate::parse;
use crate::{dependabot, workspace_version};

/// Verifies every repository surface that pins a developer tool version
/// against `.github/dev-tools.toml`.
pub fn verify_tool_pins(repo_root: &Path) -> Result<(), DevError> {
    let manifest = DevToolsManifest::load(repo_root)?;
    let mut violations = Vec::new();

    verify_rust_toolchain(repo_root, &manifest, &mut violations)?;
    verify_cargo_msrv(repo_root, &manifest, &mut violations)?;
    verify_mise_tools(repo_root, &manifest, &mut violations)?;
    verify_node_package(repo_root, &manifest, &mut violations)?;
    verify_python_uv_action(repo_root, &manifest, &mut violations)?;
    verify_rust_maintenance_action(repo_root, &manifest, &mut violations)?;
    verify_dependabot_ecosystems(repo_root, &mut violations)?;

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

fn verify_dependabot_ecosystems(
    repo_root: &Path,
    violations: &mut Vec<String>,
) -> Result<(), DevError> {
    let path = repo_root.join(".github/dependabot.yml");
    let contents = parse::read_to_string(&path)?;
    for ecosystem in [
        "github-actions",
        "pre-commit",
        "uv",
        "cargo",
        "rust-toolchain",
        "npm",
    ] {
        let expected = format!("package-ecosystem: \"{ecosystem}\"");
        if !contents.contains(&expected) {
            violations.push(format!(
                ".github/dependabot.yml update ecosystems must contain {expected:?}"
            ));
        }
    }
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
