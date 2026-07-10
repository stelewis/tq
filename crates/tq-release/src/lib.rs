mod dependabot;
mod dev_tools;
mod error;
mod external_pins;
mod runtime_deps;
mod verify;
mod workspace_version;

use std::path::Path;

pub use dev_tools::{
    DevAuditCheck, DevAuditReport, DevAuditReportStatus, DevAuditStatus, DevAuditSummary,
    DevDoctorCheck, DevDoctorReport, DevDoctorStatus, DevDoctorSummary, DevToolStatus,
};
pub use error::ReleaseError;
pub use external_pins::{
    ExternalPinReport, ExternalPinResult, ExternalPinStatus, ExternalPinSurface,
};
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
    verify_dependabot(repo_root)?;
    verify_dev_tool_pins(repo_root)
}

pub fn check_runtime_dep_changes(
    repo_root: &Path,
    base_ref: &str,
    head_ref: &str,
) -> Result<RuntimeDependencyChange, ReleaseError> {
    runtime_deps::check_runtime_dep_changes(repo_root, base_ref, head_ref)
}

pub fn verify_dev_tool_pins(repo_root: &Path) -> Result<(), ReleaseError> {
    dev_tools::verify_dev_tool_pins(repo_root)
}

pub fn audit_external_pin_drift(repo_root: &Path) -> Result<ExternalPinReport, ReleaseError> {
    external_pins::audit_external_pin_drift(repo_root)
}

pub fn doctor_dev_environment(repo_root: &Path) -> Result<DevDoctorReport, ReleaseError> {
    dev_tools::doctor_dev_environment(repo_root)
}

pub fn setup_dev_environment(repo_root: &Path) -> Result<(), ReleaseError> {
    dev_tools::setup_dev_environment(repo_root)
}

pub fn cleanup_dev_environment(repo_root: &Path) -> Result<(), ReleaseError> {
    dev_tools::cleanup_dev_environment(repo_root)
}

pub fn update_dev_dependencies(repo_root: &Path) -> Result<(), ReleaseError> {
    dev_tools::update_dev_dependencies(repo_root)
}

pub fn audit_latest_dev_dependencies(repo_root: &Path) -> Result<DevAuditReport, ReleaseError> {
    dev_tools::audit_latest_dev_dependencies(repo_root)
}

pub fn audit_security_dev_dependencies(repo_root: &Path) -> Result<DevAuditReport, ReleaseError> {
    dev_tools::audit_security_dev_dependencies(repo_root)
}

pub fn build_release_artifacts(repo_root: &Path) -> Result<(), ReleaseError> {
    dev_tools::build_release_artifacts(repo_root)
}
