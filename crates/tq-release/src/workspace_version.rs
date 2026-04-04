use std::path::{Path, PathBuf};

use toml::{Table, Value};

use crate::ReleaseError;

const ROOT_CARGO_TOML: &str = "Cargo.toml";
const CHANGELOG_PATH: &str = "CHANGELOG.md";

pub fn verify_workspace_version(repo_root: &Path) -> Result<(), ReleaseError> {
    let root_cargo_path = repo_root.join(ROOT_CARGO_TOML);
    let root_manifest = read_toml_table(&root_cargo_path, "workspace Cargo.toml")?;
    let workspace_version = workspace_version(&root_manifest, &root_cargo_path)?;
    let members = workspace_members(&root_manifest, &root_cargo_path)?;
    let workspace_dependencies = workspace_dependencies(&root_manifest, &root_cargo_path)?;

    let mut violations = Vec::new();

    for member in members {
        violations.extend(verify_member(
            repo_root,
            &member,
            workspace_version,
            workspace_dependencies,
        )?);
    }

    let changelog_path = repo_root.join(CHANGELOG_PATH);
    violations.extend(verify_changelog(&changelog_path, workspace_version)?);

    if violations.is_empty() {
        return Ok(());
    }

    Err(ReleaseError::RepositoryPolicyViolation {
        details: violations.join("\n"),
    })
}

fn verify_member(
    repo_root: &Path,
    member: &Path,
    workspace_version: &str,
    workspace_dependencies: &Table,
) -> Result<Vec<String>, ReleaseError> {
    let member_manifest_path = repo_root.join(member).join(ROOT_CARGO_TOML);
    let member_manifest = read_toml_table(&member_manifest_path, "member Cargo.toml")?;
    let package = package_table(&member_manifest, &member_manifest_path)?;
    let crate_name = package
        .get("name")
        .and_then(Value::as_str)
        .ok_or_else(|| manifest_error(&member_manifest_path, "missing package.name"))?;

    let package_version = package
        .get("version")
        .ok_or_else(|| manifest_error(&member_manifest_path, "missing package.version"))?;

    let mut violations = Vec::new();
    let uses_workspace_version = package_version
        .as_table()
        .and_then(|table| table.get("workspace"))
        .and_then(Value::as_bool)
        .unwrap_or(false);
    if !uses_workspace_version {
        violations.push(format!(
            "{crate_name} must inherit package.version from workspace.package.version"
        ));
    }

    let Some(root_dependency) = workspace_dependencies.get(crate_name) else {
        violations.push(format!(
            "workspace.dependencies is missing internal crate entry {crate_name}"
        ));
        return Ok(violations);
    };

    let Some(root_dependency_table) = root_dependency.as_table() else {
        violations.push(format!(
            "workspace.dependencies.{crate_name} must be a table with path and version"
        ));
        return Ok(violations);
    };

    let declared_version = root_dependency_table.get("version").and_then(Value::as_str);
    if declared_version != Some(workspace_version) {
        violations.push(format!(
            "workspace.dependencies.{crate_name}.version must match workspace.package.version ({workspace_version})"
        ));
    }

    let declared_path = root_dependency_table.get("path").and_then(Value::as_str);
    if declared_path.map(Path::new) != Some(member) {
        violations.push(format!(
            "workspace.dependencies.{crate_name}.path must match workspace member path {}",
            member.display()
        ));
    }

    Ok(violations)
}

fn verify_changelog(
    changelog_path: &Path,
    workspace_version: &str,
) -> Result<Vec<String>, ReleaseError> {
    let changelog_contents =
        std::fs::read_to_string(changelog_path).map_err(|source| ReleaseError::Io {
            path: changelog_path.to_path_buf(),
            source,
        })?;

    let mut violations = Vec::new();
    let expected_heading = format!("## [{workspace_version}]");
    let Some(first_release_heading) = changelog_contents
        .lines()
        .find(|line| line.starts_with("## ["))
    else {
        return Err(ReleaseError::RepositoryPolicyViolation {
            details: "CHANGELOG.md must contain at least one release heading".to_owned(),
        });
    };

    if first_release_heading != expected_heading
        && !first_release_heading.starts_with(&format!("{expected_heading} - "))
    {
        violations.push(format!(
            "CHANGELOG.md top release heading must be for workspace version {workspace_version}"
        ));
    }

    if !changelog_contents
        .lines()
        .any(|line| line == expected_heading || line.starts_with(&format!("{expected_heading} - ")))
    {
        violations.push(format!(
            "CHANGELOG.md must contain a release heading for workspace version {workspace_version}"
        ));
    }

    Ok(violations)
}

fn read_toml_table(path: &Path, description: &str) -> Result<Table, ReleaseError> {
    let contents = std::fs::read_to_string(path).map_err(|source| ReleaseError::Io {
        path: path.to_path_buf(),
        source,
    })?;

    contents
        .parse::<Table>()
        .map_err(|source| manifest_error(path, &format!("invalid {description}: {source}")))
}

fn workspace_version<'a>(manifest: &'a Table, path: &Path) -> Result<&'a str, ReleaseError> {
    manifest
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(Value::as_table)
        .and_then(|package| package.get("version"))
        .and_then(Value::as_str)
        .ok_or_else(|| manifest_error(path, "missing workspace.package.version"))
}

fn workspace_members(manifest: &Table, path: &Path) -> Result<Vec<PathBuf>, ReleaseError> {
    let members = manifest
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("members"))
        .and_then(Value::as_array)
        .ok_or_else(|| manifest_error(path, "missing workspace.members"))?;

    members
        .iter()
        .map(|member| {
            member
                .as_str()
                .map(PathBuf::from)
                .ok_or_else(|| manifest_error(path, "workspace.members entries must be strings"))
        })
        .collect()
}

fn workspace_dependencies<'a>(manifest: &'a Table, path: &Path) -> Result<&'a Table, ReleaseError> {
    manifest
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(Value::as_table)
        .ok_or_else(|| manifest_error(path, "missing workspace.dependencies"))
}

fn package_table<'a>(manifest: &'a Table, path: &Path) -> Result<&'a Table, ReleaseError> {
    manifest
        .get("package")
        .and_then(Value::as_table)
        .ok_or_else(|| manifest_error(path, "missing [package] table"))
}

fn manifest_error(path: &Path, message: &str) -> ReleaseError {
    ReleaseError::DependabotConfig {
        path: path.to_path_buf(),
        message: message.to_owned(),
    }
}
