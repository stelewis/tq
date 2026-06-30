mod dependabot;
mod error;
mod runtime_deps;
mod verify;
mod workspace_version;

use std::path::Path;

pub use error::ReleaseError;
pub use runtime_deps::RuntimeDependencyChange;
pub use verify::{ArtifactViolation, DEFAULT_FORBIDDEN_PREFIXES};

pub fn verify_artifact_contents(
    dist_dir: &Path,
    forbidden_prefixes: Option<Vec<String>>,
) -> Result<(), ReleaseError> {
    verify::verify_artifact_contents(dist_dir, forbidden_prefixes)
}

pub fn verify_dependabot(repo_root: &Path) -> Result<(), ReleaseError> {
    dependabot::verify_dependabot(repo_root)
}

pub fn verify_workspace_version(repo_root: &Path) -> Result<(), ReleaseError> {
    workspace_version::verify_workspace_version(repo_root)
}

pub fn verify_release_policy(repo_root: &Path) -> Result<(), ReleaseError> {
    verify_workspace_version(repo_root)?;
    verify_dependabot(repo_root)
}

pub fn check_runtime_dep_changes(
    repo_root: &Path,
    base_ref: &str,
    head_ref: &str,
) -> Result<RuntimeDependencyChange, ReleaseError> {
    runtime_deps::check_runtime_dep_changes(repo_root, base_ref, head_ref)
}
