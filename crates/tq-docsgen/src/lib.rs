mod cli_docs;
mod config_examples;
mod error;
mod markers;
mod rules_docs;

use std::path::Path;

pub use error::DocsgenError;

pub fn generate_all(workspace_root: &Path) -> Result<(), DocsgenError> {
    generate_cli_docs(workspace_root)?;
    generate_config_examples(workspace_root)?;
    generate_rules_docs(workspace_root)?;
    Ok(())
}

pub fn generate_cli_docs(workspace_root: &Path) -> Result<(), DocsgenError> {
    cli_docs::generate(workspace_root)
}

pub fn generate_config_examples(workspace_root: &Path) -> Result<(), DocsgenError> {
    config_examples::generate(workspace_root)
}

pub fn generate_rules_docs(workspace_root: &Path) -> Result<(), DocsgenError> {
    rules_docs::generate(workspace_root)
}
