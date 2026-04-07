use std::path::Path;

use toml::{Table, Value};

use crate::{DocsgenError, markers::replace_between_markers};

const ROOT_CARGO_TOML_PATH: &str = "Cargo.toml";
const PYPROJECT_TOML_PATH: &str = "pyproject.toml";
const README_PATH: &str = "README.md";
const QUICKSTART_PATH: &str = "docs/guide/quickstart.md";
const PRE_COMMIT_START: &str = "<!-- BEGIN GENERATED:pre-commit-config -->";
const PRE_COMMIT_END: &str = "<!-- END GENERATED:pre-commit-config -->";

pub fn generate(workspace_root: &Path) -> Result<(), DocsgenError> {
    let cargo_toml_path = workspace_root.join(ROOT_CARGO_TOML_PATH);
    let pyproject_toml_path = workspace_root.join(PYPROJECT_TOML_PATH);
    let readme_path = workspace_root.join(README_PATH);
    let quickstart_path = workspace_root.join(QUICKSTART_PATH);

    let workspace_version = workspace_version(&cargo_toml_path)?;
    let tag_format = tag_format(&pyproject_toml_path)?;
    let release_ref = tag_format.replace("$version", &workspace_version);
    let replacement = render_pre_commit_block(&release_ref);

    replace_between_markers(&readme_path, PRE_COMMIT_START, PRE_COMMIT_END, &replacement)?;
    replace_between_markers(
        &quickstart_path,
        PRE_COMMIT_START,
        PRE_COMMIT_END,
        &replacement,
    )
}

fn workspace_version(path: &Path) -> Result<String, DocsgenError> {
    let manifest = read_toml_table(path)?;
    let version = manifest
        .get("workspace")
        .and_then(Value::as_table)
        .and_then(|workspace| workspace.get("package"))
        .and_then(Value::as_table)
        .and_then(|package| package.get("version"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            DocsgenError::manifest(path.to_path_buf(), "missing workspace.package.version")
        })?;

    if version.trim().is_empty() {
        return Err(DocsgenError::manifest(
            path.to_path_buf(),
            "workspace.package.version must be non-empty",
        ));
    }

    Ok(version.to_owned())
}

fn tag_format(path: &Path) -> Result<String, DocsgenError> {
    let manifest = read_toml_table(path)?;
    let tag_format = manifest
        .get("tool")
        .and_then(Value::as_table)
        .and_then(|tool| tool.get("commitizen"))
        .and_then(Value::as_table)
        .and_then(|commitizen| commitizen.get("tag_format"))
        .and_then(Value::as_str)
        .ok_or_else(|| {
            DocsgenError::manifest(path.to_path_buf(), "missing tool.commitizen.tag_format")
        })?;

    if tag_format.trim().is_empty() {
        return Err(DocsgenError::manifest(
            path.to_path_buf(),
            "tool.commitizen.tag_format must be non-empty",
        ));
    }

    Ok(tag_format.to_owned())
}

fn read_toml_table(path: &Path) -> Result<Table, DocsgenError> {
    let content = std::fs::read_to_string(path).map_err(|source| DocsgenError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&content).map_err(|source| {
        DocsgenError::manifest(
            path.to_path_buf(),
            format!("failed to parse TOML: {source}"),
        )
    })
}

fn render_pre_commit_block(release_ref: &str) -> String {
    format!(
        "```yaml\nrepos:\n  - repo: https://github.com/stelewis/tq\n    rev: {release_ref}\n    hooks:\n      - id: tq-check\n```"
    )
}
