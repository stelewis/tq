use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;

use crate::ReleaseError;

pub const DEFAULT_FORBIDDEN_PREFIXES: &[&str] =
    &["scripts/", "tests/", "docs/", "tmp/", ".github/", "src/tq/"];

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactViolation {
    pub artifact: PathBuf,
    pub member: String,
}

pub fn verify_artifact_contents(
    dist_dir: &Path,
    forbidden_prefixes: Option<Vec<String>>,
) -> Result<(), ReleaseError> {
    if !dist_dir.exists() {
        return Err(ReleaseError::MissingDistributionDirectory {
            path: dist_dir.to_path_buf(),
        });
    }

    let forbidden_prefixes = forbidden_prefixes.unwrap_or_else(|| {
        DEFAULT_FORBIDDEN_PREFIXES
            .iter()
            .map(|prefix| (*prefix).to_owned())
            .collect()
    });
    let violations = collect_violations(dist_dir, &forbidden_prefixes)?;
    if violations.is_empty() {
        return Ok(());
    }

    let details = violations
        .iter()
        .map(|violation| format!("- {}: {}", violation.artifact.display(), violation.member))
        .collect::<Vec<_>>()
        .join("\n");
    Err(ReleaseError::PolicyViolation { details })
}

fn collect_violations(
    dist_dir: &Path,
    forbidden_prefixes: &[String],
) -> Result<Vec<ArtifactViolation>, ReleaseError> {
    let mut violations = Vec::new();

    for entry in std::fs::read_dir(dist_dir).map_err(|source| ReleaseError::Io {
        path: dist_dir.to_path_buf(),
        source,
    })? {
        let entry = entry.map_err(|source| ReleaseError::Io {
            path: dist_dir.to_path_buf(),
            source,
        })?;
        let artifact_path = entry.path();
        if artifact_path.is_dir() {
            continue;
        }

        if has_extension(&artifact_path, "zip") || has_extension(&artifact_path, "whl") {
            violations.extend(find_zip_violations(&artifact_path, forbidden_prefixes)?);
        } else if is_tar_gz(&artifact_path) {
            violations.extend(find_tar_gz_violations(&artifact_path, forbidden_prefixes)?);
        }
    }

    Ok(violations)
}

fn find_zip_violations(
    artifact_path: &Path,
    forbidden_prefixes: &[String],
) -> Result<Vec<ArtifactViolation>, ReleaseError> {
    let file = std::fs::File::open(artifact_path).map_err(|source| ReleaseError::Io {
        path: artifact_path.to_path_buf(),
        source,
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|source| ReleaseError::Zip {
        path: artifact_path.to_path_buf(),
        source,
    })?;
    let mut violations = Vec::new();

    for index in 0..archive.len() {
        let member = archive
            .by_index(index)
            .map_err(|source| ReleaseError::Zip {
                path: artifact_path.to_path_buf(),
                source,
            })?;
        let member_name = member.name().to_owned();
        if is_forbidden_member(&member_name, forbidden_prefixes) {
            violations.push(ArtifactViolation {
                artifact: artifact_path.to_path_buf(),
                member: member_name,
            });
        }
    }

    Ok(violations)
}

fn find_tar_gz_violations(
    artifact_path: &Path,
    forbidden_prefixes: &[String],
) -> Result<Vec<ArtifactViolation>, ReleaseError> {
    let file = std::fs::File::open(artifact_path).map_err(|source| ReleaseError::Io {
        path: artifact_path.to_path_buf(),
        source,
    })?;
    let decoder = GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    let mut violations = Vec::new();

    let entries = archive.entries().map_err(|source| ReleaseError::Io {
        path: artifact_path.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let mut entry = entry.map_err(|source| ReleaseError::Io {
            path: artifact_path.to_path_buf(),
            source,
        })?;
        let mut sink = Vec::new();
        let _ = entry.read_to_end(&mut sink);
        let member_name = entry
            .path()
            .map_err(|source| ReleaseError::Io {
                path: artifact_path.to_path_buf(),
                source,
            })?
            .display()
            .to_string();
        if is_forbidden_member(&member_name, forbidden_prefixes) {
            violations.push(ArtifactViolation {
                artifact: artifact_path.to_path_buf(),
                member: member_name,
            });
        }
    }

    Ok(violations)
}

fn is_forbidden_member(member_name: &str, forbidden_prefixes: &[String]) -> bool {
    if member_name.contains(".data/scripts/") {
        return normalized_candidates(member_name).iter().any(|candidate| {
            forbidden_prefixes
                .iter()
                .any(|prefix| prefix != "scripts/" && candidate.starts_with(prefix))
        });
    }

    normalized_candidates(member_name).iter().any(|candidate| {
        forbidden_prefixes
            .iter()
            .any(|prefix| candidate.starts_with(prefix))
    })
}

fn normalized_candidates(member_name: &str) -> Vec<String> {
    let trimmed = member_name.trim_start_matches("./");
    let mut candidates = vec![trimmed.to_owned()];

    if let Some((_, remainder)) = trimmed.split_once('/') {
        candidates.push(remainder.to_owned());
    }

    candidates
}

fn has_extension(path: &Path, extension: &str) -> bool {
    path.extension()
        .is_some_and(|value| value.eq_ignore_ascii_case(extension))
}

fn is_tar_gz(path: &Path) -> bool {
    path.file_name()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|value| value.ends_with(".tar.gz"))
}
