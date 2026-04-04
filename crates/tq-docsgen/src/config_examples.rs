use std::path::Path;

use serde::Deserialize;

use crate::{DocsgenError, markers::replace_between_markers};

const MANIFEST_PATH: &str = "docs/reference/config/examples-manifest.json";
const QUICKSTART_PATH: &str = "docs/guide/quickstart.md";
const CONFIGURATION_PATH: &str = "docs/reference/configuration.md";

const QUICKSTART_MINIMAL_START: &str = "<!-- BEGIN GENERATED:quickstart-minimal-config -->";
const QUICKSTART_MINIMAL_END: &str = "<!-- END GENERATED:quickstart-minimal-config -->";
const CONFIGURATION_MINIMAL_START: &str = "<!-- BEGIN GENERATED:configuration-minimal-config -->";
const CONFIGURATION_MINIMAL_END: &str = "<!-- END GENERATED:configuration-minimal-config -->";
const CONFIGURATION_TYPICAL_START: &str = "<!-- BEGIN GENERATED:configuration-typical-config -->";
const CONFIGURATION_TYPICAL_END: &str = "<!-- END GENERATED:configuration-typical-config -->";

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigExamplesManifest {
    version: u64,
    examples: ConfigExamples,
}

#[derive(Debug, Deserialize)]
#[serde(deny_unknown_fields)]
struct ConfigExamples {
    quickstart_minimal: String,
    configuration_minimal: String,
    configuration_typical: String,
}

pub fn generate(workspace_root: &Path) -> Result<(), DocsgenError> {
    let manifest_path = workspace_root.join(MANIFEST_PATH);
    let quickstart_path = workspace_root.join(QUICKSTART_PATH);
    let configuration_path = workspace_root.join(CONFIGURATION_PATH);
    let manifest = load_manifest(&manifest_path)?;

    replace_between_markers(
        &quickstart_path,
        QUICKSTART_MINIMAL_START,
        QUICKSTART_MINIMAL_END,
        &render_toml_block(&manifest.quickstart_minimal),
    )?;
    replace_between_markers(
        &configuration_path,
        CONFIGURATION_MINIMAL_START,
        CONFIGURATION_MINIMAL_END,
        &render_toml_block(&manifest.configuration_minimal),
    )?;
    replace_between_markers(
        &configuration_path,
        CONFIGURATION_TYPICAL_START,
        CONFIGURATION_TYPICAL_END,
        &render_toml_block(&manifest.configuration_typical),
    )
}

fn load_manifest(path: &Path) -> Result<ConfigExamples, DocsgenError> {
    let content = std::fs::read_to_string(path).map_err(|source| DocsgenError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let manifest: ConfigExamplesManifest =
        serde_json::from_str(&content).map_err(|source| DocsgenError::Json {
            path: path.to_path_buf(),
            source,
        })?;

    if manifest.version != 1 {
        return Err(DocsgenError::manifest(
            path.to_path_buf(),
            format!("unsupported manifest version: {}", manifest.version),
        ));
    }

    validate_non_empty(
        path,
        "examples.quickstart_minimal",
        &manifest.examples.quickstart_minimal,
    )?;
    validate_non_empty(
        path,
        "examples.configuration_minimal",
        &manifest.examples.configuration_minimal,
    )?;
    validate_non_empty(
        path,
        "examples.configuration_typical",
        &manifest.examples.configuration_typical,
    )?;

    Ok(manifest.examples)
}

fn validate_non_empty(path: &Path, field_name: &str, value: &str) -> Result<(), DocsgenError> {
    if value.trim().is_empty() {
        return Err(DocsgenError::manifest(
            path.to_path_buf(),
            format!("{field_name} must be non-empty text"),
        ));
    }

    Ok(())
}

fn render_toml_block(snippet: &str) -> String {
    format!("```toml\n{}\n```", snippet.trim())
}
