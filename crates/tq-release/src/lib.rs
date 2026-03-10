mod error;
mod verify;

use std::path::Path;

pub use error::ReleaseError;
pub use verify::{ArtifactViolation, DEFAULT_FORBIDDEN_PREFIXES};

pub fn verify_artifact_contents(
    dist_dir: &Path,
    forbidden_prefixes: Option<Vec<String>>,
) -> Result<(), ReleaseError> {
    verify::verify_artifact_contents(dist_dir, forbidden_prefixes)
}
