use std::path::{Path, PathBuf};

use toml::{Table, Value};

use crate::ReleaseError;

const ROOT_CARGO_TOML: &str = "Cargo.toml";
const CHANGELOG_PATH: &str = "CHANGELOG.md";

pub fn sync_workspace_dependency_versions(repo_root: &Path) -> Result<(), ReleaseError> {
    let root_cargo_path = repo_root.join(ROOT_CARGO_TOML);
    let root_manifest = read_toml_table(&root_cargo_path, "workspace Cargo.toml")?;
    let workspace_version = workspace_version(&root_manifest, &root_cargo_path)?;
    let members = workspace_members(&root_manifest, &root_cargo_path)?;
    let internal_crate_names = internal_crate_names(repo_root, &members)?;
    let updated = sync_dependency_versions_in_manifest(
        &std::fs::read_to_string(&root_cargo_path).map_err(|source| ReleaseError::Io {
            path: root_cargo_path.clone(),
            source,
        })?,
        &internal_crate_names,
        workspace_version,
        &root_cargo_path,
    )?;

    std::fs::write(&root_cargo_path, updated).map_err(|source| ReleaseError::Io {
        path: root_cargo_path,
        source,
    })
}

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

fn sync_dependency_versions_in_manifest(
    cargo_toml: &str,
    internal_crate_names: &[String],
    workspace_version: &str,
    path: &Path,
) -> Result<String, ReleaseError> {
    let mut updated_lines = Vec::new();
    let mut in_workspace_dependencies = false;
    let mut seen_crate_names = Vec::new();

    for line in cargo_toml.split_inclusive('\n') {
        let mut updated_line = line.to_owned();
        let trimmed = line.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_workspace_dependencies = trimmed == "[workspace.dependencies]";
        } else if in_workspace_dependencies {
            for crate_name in internal_crate_names {
                if line.starts_with(&format!("{crate_name} = {{")) {
                    let replacement = format!("version = \"{workspace_version}\"");
                    if let Some((prefix, suffix)) = updated_line.split_once("version = \"") {
                        let Some((_, tail)) = suffix.split_once('"') else {
                            return Err(workspace_version_error(
                                path,
                                &format!(
                                    "workspace.dependencies.{crate_name} is missing a quoted version"
                                ),
                            ));
                        };
                        updated_line = format!("{prefix}{replacement}{tail}");
                    } else {
                        return Err(workspace_version_error(
                            path,
                            &format!("workspace.dependencies.{crate_name} is missing version"),
                        ));
                    }
                    seen_crate_names.push(crate_name.clone());
                    break;
                }
            }
        }

        updated_lines.push(updated_line);
    }

    let missing_crate_names: Vec<_> = internal_crate_names
        .iter()
        .filter(|crate_name| !seen_crate_names.contains(crate_name))
        .cloned()
        .collect();

    if !missing_crate_names.is_empty() {
        return Err(workspace_version_error(
            path,
            &format!(
                "workspace.dependencies is missing internal crate entries: {}",
                missing_crate_names.join(", ")
            ),
        ));
    }

    Ok(updated_lines.concat())
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
        .ok_or_else(|| workspace_version_error(&member_manifest_path, "missing package.name"))?;

    let package_version = package
        .get("version")
        .ok_or_else(|| workspace_version_error(&member_manifest_path, "missing package.version"))?;

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

fn internal_crate_names(
    repo_root: &Path,
    members: &[PathBuf],
) -> Result<Vec<String>, ReleaseError> {
    members
        .iter()
        .map(|member| {
            let member_manifest_path = repo_root.join(member).join(ROOT_CARGO_TOML);
            let member_manifest = read_toml_table(&member_manifest_path, "member Cargo.toml")?;
            let package = package_table(&member_manifest, &member_manifest_path)?;
            package
                .get("name")
                .and_then(Value::as_str)
                .map(str::to_owned)
                .ok_or_else(|| {
                    workspace_version_error(&member_manifest_path, "missing package.name")
                })
        })
        .collect()
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

    contents.parse::<Table>().map_err(|source| {
        workspace_version_error(path, &format!("invalid {description}: {source}"))
    })
}

fn workspace_version<'a>(manifest: &'a Table, path: &Path) -> Result<&'a str, ReleaseError> {
    manifest
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(Value::as_table)
        .and_then(|package| package.get("version"))
        .and_then(Value::as_str)
        .ok_or_else(|| workspace_version_error(path, "missing workspace.package.version"))
}

fn workspace_members(manifest: &Table, path: &Path) -> Result<Vec<PathBuf>, ReleaseError> {
    let members = manifest
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("members"))
        .and_then(Value::as_array)
        .ok_or_else(|| workspace_version_error(path, "missing workspace.members"))?;

    members
        .iter()
        .map(|member| {
            member.as_str().map(PathBuf::from).ok_or_else(|| {
                workspace_version_error(path, "workspace.members entries must be strings")
            })
        })
        .collect()
}

fn workspace_dependencies<'a>(manifest: &'a Table, path: &Path) -> Result<&'a Table, ReleaseError> {
    manifest
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("dependencies"))
        .and_then(Value::as_table)
        .ok_or_else(|| workspace_version_error(path, "missing workspace.dependencies"))
}

fn package_table<'a>(manifest: &'a Table, path: &Path) -> Result<&'a Table, ReleaseError> {
    manifest
        .get("package")
        .and_then(Value::as_table)
        .ok_or_else(|| workspace_version_error(path, "missing [package] table"))
}

fn workspace_version_error(path: &Path, message: &str) -> ReleaseError {
    ReleaseError::WorkspaceVersionInput {
        path: path.to_path_buf(),
        message: message.to_owned(),
    }
}
