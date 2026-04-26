mod dependabot;
mod error;
mod release_intent;
mod release_intent_repo;
mod verify;
mod workspace_version;

use std::path::{Path, PathBuf};

pub use error::ReleaseError;
pub use release_intent::{RELEASE_INTENT_LABELS, ReleaseIntent};
pub use verify::{ArtifactViolation, DEFAULT_FORBIDDEN_PREFIXES};

#[derive(Clone, Copy, Debug)]
pub struct PrReleaseIntentCheck<'a> {
    pub repo_root: &'a Path,
    pub base_ref: &'a str,
    pub head_ref: &'a str,
    pub labels: &'a [String],
}

#[derive(Clone, Copy, Debug)]
pub struct ReleaseIntentCheck<'a> {
    pub labels: &'a [String],
    pub changed_files: &'a [PathBuf],
    pub version_updated: bool,
    pub changelog_updated: bool,
    pub runtime_dependency_changed: bool,
}

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

pub fn verify_release_intent(input: ReleaseIntentCheck<'_>) -> Result<(), ReleaseError> {
    release_intent::verify_release_intent(input)
}

pub fn verify_pr_release_intent(input: PrReleaseIntentCheck<'_>) -> Result<(), ReleaseError> {
    release_intent_repo::verify_pr_release_intent(input)
}
