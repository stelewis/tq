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
    let workspace_dependencies = workspace_dependencies(&root_manifest, &root_cargo_path)?;
    let internal_crate_names = internal_crate_names(repo_root, &members)?;
    validate_internal_workspace_dependency_entries(
        workspace_dependencies,
        &internal_crate_names,
        &root_cargo_path,
    )?;
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
    let mut dependency_table_crate_name: Option<String> = None;
    let mut seen_crate_names = Vec::new();

    for line in cargo_toml.split_inclusive('\n') {
        let mut updated_line = line.to_owned();
        let trimmed = line.trim();

        if trimmed.starts_with('[') && trimmed.ends_with(']') {
            in_workspace_dependencies = trimmed == "[workspace.dependencies]";
            dependency_table_crate_name = trimmed
                .strip_prefix("[workspace.dependencies.")
                .and_then(|value| value.strip_suffix(']'))
                .filter(|crate_name| internal_crate_names.iter().any(|name| name == crate_name))
                .map(str::to_owned);
        } else if let Some(crate_name) = dependency_table_crate_name.as_deref() {
            if trimmed.starts_with("version") {
                updated_line = replace_table_dependency_version(
                    &updated_line,
                    crate_name,
                    workspace_version,
                    path,
                )?;
                seen_crate_names.push(crate_name.to_owned());
            }
        } else if in_workspace_dependencies {
            for crate_name in internal_crate_names {
                if line.trim_start().starts_with(&format!("{crate_name} = {{")) {
                    updated_line = replace_inline_dependency_version(
                        &updated_line,
                        crate_name,
                        workspace_version,
                        path,
                    )?;
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
                "workspace.dependencies internal crate entries use unsupported manifest formatting: {}",
                missing_crate_names.join(", ")
            ),
        ));
    }

    Ok(updated_lines.concat())
}

fn validate_internal_workspace_dependency_entries(
    workspace_dependencies: &Table,
    internal_crate_names: &[String],
    path: &Path,
) -> Result<(), ReleaseError> {
    let missing_crate_names: Vec<_> = internal_crate_names
        .iter()
        .filter(|crate_name| !workspace_dependencies.contains_key(crate_name.as_str()))
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

    for crate_name in internal_crate_names {
        let dependency = workspace_dependencies
            .get(crate_name)
            .expect("validated internal crate entry must exist");
        let dependency_table = dependency.as_table().ok_or_else(|| {
            workspace_version_error(
                path,
                &format!(
                    "workspace.dependencies.{crate_name} must be a table with path and version"
                ),
            )
        })?;

        if dependency_table
            .get("version")
            .and_then(Value::as_str)
            .is_none()
        {
            return Err(workspace_version_error(
                path,
                &format!("workspace.dependencies.{crate_name} is missing version"),
            ));
        }
    }

    Ok(())
}

fn replace_inline_dependency_version(
    line: &str,
    crate_name: &str,
    workspace_version: &str,
    path: &Path,
) -> Result<String, ReleaseError> {
    replace_dependency_version_value(line, crate_name, workspace_version, path, "version")
}

fn replace_table_dependency_version(
    line: &str,
    crate_name: &str,
    workspace_version: &str,
    path: &Path,
) -> Result<String, ReleaseError> {
    replace_dependency_version_value(line, crate_name, workspace_version, path, "version")
}

fn replace_dependency_version_value(
    line: &str,
    crate_name: &str,
    workspace_version: &str,
    path: &Path,
    key: &str,
) -> Result<String, ReleaseError> {
    let Some(key_index) = find_key_assignment(line, key) else {
        return Err(workspace_version_error(
            path,
            &format!("workspace.dependencies.{crate_name} is missing version"),
        ));
    };
    let value_start = key_index + key.len();
    let Some(equals_index_offset) = line[value_start..].find('=') else {
        return Err(workspace_version_error(
            path,
            &format!("workspace.dependencies.{crate_name} is missing version"),
        ));
    };
    let value_search_start = value_start + equals_index_offset + 1;
    let Some(open_quote_offset) = line[value_search_start..].find('"') else {
        return Err(workspace_version_error(
            path,
            &format!("workspace.dependencies.{crate_name} is missing a quoted version"),
        ));
    };
    let open_quote_index = value_search_start + open_quote_offset;
    let Some(close_quote_offset) = line[open_quote_index + 1..].find('"') else {
        return Err(workspace_version_error(
            path,
            &format!("workspace.dependencies.{crate_name} is missing a quoted version"),
        ));
    };
    let close_quote_index = open_quote_index + 1 + close_quote_offset;

    Ok(format!(
        "{}\"{}\"{}",
        &line[..open_quote_index],
        workspace_version,
        &line[close_quote_index + 1..]
    ))
}

fn find_key_assignment(line: &str, key: &str) -> Option<usize> {
    line.match_indices(key).find_map(|(index, _)| {
        let before = line[..index].chars().next_back();
        let after = line[index + key.len()..].chars().next();
        let before_ok = before.is_none_or(|character| matches!(character, ' ' | '\t' | '{' | ','));
        let after_ok = after.is_none_or(|character| matches!(character, ' ' | '\t' | '='));

        before_ok.then_some(())?;
        after_ok.then_some(index)
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
