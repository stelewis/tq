use std::collections::BTreeMap;
use std::io::Read;
use std::path::{Path, PathBuf};

use flate2::read::GzDecoder;

use crate::error::DevError;

pub const DEFAULT_FORBIDDEN_PREFIXES: &[&str] = &[
    "scripts/", "tests/", "docs/", "tmp/", ".github/", ".vscode/",
];

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum WheelPlatform {
    PortableLinux,
    MacosX86_64,
    MacosArm64,
    WindowsX86_64,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ArtifactExpectation {
    SdistOnly,
    WheelOnly(WheelPlatform),
    SdistAndWheel(WheelPlatform),
    FullRelease,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ArtifactKind {
    Sdist,
    Wheel(WheelPlatform),
    NativeLinuxWheel,
    UnsupportedWheel,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Artifact {
    pub path: PathBuf,
    pub kind: ArtifactKind,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ArtifactViolation {
    UnexpectedPath {
        path: PathBuf,
    },
    UnexpectedKind {
        artifact: Artifact,
    },
    Count {
        kind: ArtifactKind,
        expected: usize,
        actual: usize,
    },
    NativeLinuxTag {
        path: PathBuf,
    },
    ForbiddenMember {
        artifact: PathBuf,
        member: String,
    },
    MissingDeclaredLicense {
        artifact: PathBuf,
        license: String,
    },
}

impl ArtifactViolation {
    fn describe(&self) -> String {
        match self {
            Self::UnexpectedPath { path } => {
                format!("unexpected distribution path: {}", path.display())
            }
            Self::UnexpectedKind { artifact } => format!(
                "unexpected artifact kind {:?}: {}",
                artifact.kind,
                artifact.path.display()
            ),
            Self::Count {
                kind,
                expected,
                actual,
            } => format!("expected {expected} {kind:?} artifact(s), found {actual}"),
            Self::NativeLinuxTag { path } => format!(
                "wheel uses a native Linux platform tag rejected by PyPI: {}",
                path.display()
            ),
            Self::ForbiddenMember { artifact, member } => {
                format!("{}: forbidden archive member {member}", artifact.display())
            }
            Self::MissingDeclaredLicense { artifact, license } => format!(
                "{}: missing declared license file: {license}",
                artifact.display()
            ),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ArtifactReport {
    pub artifacts: Vec<Artifact>,
    pub violations: Vec<ArtifactViolation>,
}

pub fn verify_artifacts(
    dist_dir: &Path,
    expectation: ArtifactExpectation,
    forbidden_prefixes: Option<Vec<String>>,
) -> Result<ArtifactReport, DevError> {
    if !dist_dir.is_dir() {
        return Err(DevError::MissingDistributionDirectory {
            path: dist_dir.to_path_buf(),
        });
    }

    let forbidden_prefixes = forbidden_prefixes.unwrap_or_else(|| {
        DEFAULT_FORBIDDEN_PREFIXES
            .iter()
            .map(|prefix| (*prefix).to_owned())
            .collect()
    });
    let mut report = collect_report(dist_dir, &forbidden_prefixes)?;
    validate_artifact_set(&mut report, expectation);
    if report.violations.is_empty() {
        return Ok(report);
    }

    let details = report
        .violations
        .iter()
        .map(|violation| format!("- {}", violation.describe()))
        .collect::<Vec<_>>()
        .join("\n");
    Err(DevError::ArtifactPolicyViolation { details })
}

fn collect_report(
    dist_dir: &Path,
    forbidden_prefixes: &[String],
) -> Result<ArtifactReport, DevError> {
    let mut artifacts = Vec::new();
    let mut violations = Vec::new();
    let mut entries = std::fs::read_dir(dist_dir)
        .map_err(|source| DevError::Io {
            path: dist_dir.to_path_buf(),
            source,
        })?
        .collect::<Result<Vec<_>, _>>()
        .map_err(|source| DevError::Io {
            path: dist_dir.to_path_buf(),
            source,
        })?;
    entries.sort_by_key(std::fs::DirEntry::path);

    for entry in entries {
        let artifact_path = entry.path();
        if artifact_path
            .file_name()
            .is_some_and(|name| name == ".gitignore")
        {
            continue;
        }
        let Some(kind) = classify_artifact(&artifact_path) else {
            violations.push(ArtifactViolation::UnexpectedPath {
                path: artifact_path,
            });
            continue;
        };
        let artifact = Artifact {
            path: artifact_path.clone(),
            kind,
        };
        artifacts.push(artifact);

        if matches!(kind, ArtifactKind::NativeLinuxWheel) {
            violations.push(ArtifactViolation::NativeLinuxTag {
                path: artifact_path.clone(),
            });
        }
        if has_extension(&artifact_path, "whl") {
            violations.extend(find_zip_violations(&artifact_path, forbidden_prefixes)?);
        } else if is_tar_gz(&artifact_path) {
            violations.extend(find_tar_gz_violations(&artifact_path, forbidden_prefixes)?);
        }
    }

    Ok(ArtifactReport {
        artifacts,
        violations,
    })
}

fn classify_artifact(path: &Path) -> Option<ArtifactKind> {
    if is_tar_gz(path) {
        return Some(ArtifactKind::Sdist);
    }
    if !has_extension(path, "whl") {
        return None;
    }
    let name = path.file_name()?.to_str()?.to_ascii_lowercase();
    Some(
        if name.contains("manylinux") || name.contains("musllinux") {
            ArtifactKind::Wheel(WheelPlatform::PortableLinux)
        } else if name.contains("linux") {
            ArtifactKind::NativeLinuxWheel
        } else if name.contains("macosx") && name.ends_with("x86_64.whl") {
            ArtifactKind::Wheel(WheelPlatform::MacosX86_64)
        } else if name.contains("macosx") && name.ends_with("arm64.whl") {
            ArtifactKind::Wheel(WheelPlatform::MacosArm64)
        } else if name.ends_with("win_amd64.whl") {
            ArtifactKind::Wheel(WheelPlatform::WindowsX86_64)
        } else {
            ArtifactKind::UnsupportedWheel
        },
    )
}

fn validate_artifact_set(report: &mut ArtifactReport, expectation: ArtifactExpectation) {
    let expected = expected_counts(expectation);
    let mut actual = BTreeMap::new();
    for artifact in &report.artifacts {
        *actual.entry(artifact.kind).or_insert(0) += 1;
        if !expected.contains_key(&artifact.kind) {
            report.violations.push(ArtifactViolation::UnexpectedKind {
                artifact: artifact.clone(),
            });
        }
    }
    for (kind, expected_count) in expected {
        let actual_count = actual.get(&kind).copied().unwrap_or(0);
        if actual_count != expected_count {
            report.violations.push(ArtifactViolation::Count {
                kind,
                expected: expected_count,
                actual: actual_count,
            });
        }
    }
}

fn expected_counts(expectation: ArtifactExpectation) -> BTreeMap<ArtifactKind, usize> {
    let kinds = match expectation {
        ArtifactExpectation::SdistOnly => vec![ArtifactKind::Sdist],
        ArtifactExpectation::WheelOnly(platform) => vec![ArtifactKind::Wheel(platform)],
        ArtifactExpectation::SdistAndWheel(platform) => {
            vec![ArtifactKind::Sdist, ArtifactKind::Wheel(platform)]
        }
        ArtifactExpectation::FullRelease => vec![
            ArtifactKind::Sdist,
            ArtifactKind::Wheel(WheelPlatform::PortableLinux),
            ArtifactKind::Wheel(WheelPlatform::MacosX86_64),
            ArtifactKind::Wheel(WheelPlatform::MacosArm64),
            ArtifactKind::Wheel(WheelPlatform::WindowsX86_64),
        ],
    };
    kinds.into_iter().map(|kind| (kind, 1)).collect()
}

fn find_zip_violations(
    artifact_path: &Path,
    forbidden_prefixes: &[String],
) -> Result<Vec<ArtifactViolation>, DevError> {
    let file = std::fs::File::open(artifact_path).map_err(|source| DevError::Io {
        path: artifact_path.to_path_buf(),
        source,
    })?;
    let mut archive = zip::ZipArchive::new(file).map_err(|source| DevError::Zip {
        path: artifact_path.to_path_buf(),
        source,
    })?;
    let mut violations = Vec::new();
    let mut members = Vec::with_capacity(archive.len());
    let mut declared_license_files = Vec::new();

    for index in 0..archive.len() {
        let mut member = archive.by_index(index).map_err(|source| DevError::Zip {
            path: artifact_path.to_path_buf(),
            source,
        })?;
        let member_name = member.name().to_owned();
        members.push(member_name.clone());
        if is_forbidden_member(&member_name, forbidden_prefixes) {
            violations.push(ArtifactViolation::ForbiddenMember {
                artifact: artifact_path.to_path_buf(),
                member: member_name.clone(),
            });
        }

        if member_name.ends_with(".dist-info/METADATA") {
            let metadata = read_zip_member_to_string(&mut member, artifact_path)?;
            declared_license_files.extend(parse_declared_license_files(&metadata));
        }
    }

    violations.extend(find_missing_declared_license_files(
        artifact_path,
        &members,
        &declared_license_files,
    ));

    Ok(violations)
}

fn find_tar_gz_violations(
    artifact_path: &Path,
    forbidden_prefixes: &[String],
) -> Result<Vec<ArtifactViolation>, DevError> {
    let file = std::fs::File::open(artifact_path).map_err(|source| DevError::Io {
        path: artifact_path.to_path_buf(),
        source,
    })?;
    let decoder = GzDecoder::new(file);
    let mut archive = tar::Archive::new(decoder);
    let mut violations = Vec::new();
    let mut members = Vec::new();
    let mut declared_license_files = Vec::new();

    let entries = archive.entries().map_err(|source| DevError::Io {
        path: artifact_path.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let mut entry = entry.map_err(|source| DevError::Io {
            path: artifact_path.to_path_buf(),
            source,
        })?;
        let member_name = entry
            .path()
            .map_err(|source| DevError::Io {
                path: artifact_path.to_path_buf(),
                source,
            })?
            .display()
            .to_string();
        members.push(member_name.clone());
        if is_forbidden_member(&member_name, forbidden_prefixes) {
            violations.push(ArtifactViolation::ForbiddenMember {
                artifact: artifact_path.to_path_buf(),
                member: member_name.clone(),
            });
        }

        if member_name.ends_with("/PKG-INFO") || member_name == "PKG-INFO" {
            let metadata = read_tar_member_to_string(&mut entry, artifact_path)?;
            declared_license_files.extend(parse_declared_license_files(&metadata));
        }
    }

    violations.extend(find_missing_declared_license_files(
        artifact_path,
        &members,
        &declared_license_files,
    ));

    Ok(violations)
}

fn read_zip_member_to_string<R: Read>(
    member: &mut R,
    artifact_path: &Path,
) -> Result<String, DevError> {
    let mut metadata = String::new();
    member
        .read_to_string(&mut metadata)
        .map_err(|source| DevError::Io {
            path: artifact_path.to_path_buf(),
            source,
        })?;
    Ok(metadata)
}

fn read_tar_member_to_string<R: Read>(
    member: &mut R,
    artifact_path: &Path,
) -> Result<String, DevError> {
    let mut metadata = String::new();
    member
        .read_to_string(&mut metadata)
        .map_err(|source| DevError::Io {
            path: artifact_path.to_path_buf(),
            source,
        })?;
    Ok(metadata)
}

fn parse_declared_license_files(metadata: &str) -> Vec<String> {
    metadata
        .lines()
        .filter_map(|line| line.strip_prefix("License-File: "))
        .map(str::trim)
        .filter(|value| !value.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}

fn find_missing_declared_license_files(
    artifact_path: &Path,
    members: &[String],
    declared_license_files: &[String],
) -> Vec<ArtifactViolation> {
    declared_license_files
        .iter()
        .filter(|license_file| {
            !members
                .iter()
                .any(|member| member_matches_declared_license_file(member, license_file))
        })
        .map(|license_file| ArtifactViolation::MissingDeclaredLicense {
            artifact: artifact_path.to_path_buf(),
            license: license_file.clone(),
        })
        .collect()
}

fn member_matches_declared_license_file(member_name: &str, license_file: &str) -> bool {
    metadata_member_candidates(member_name)
        .iter()
        .any(|candidate| candidate == license_file)
}

fn metadata_member_candidates(member_name: &str) -> Vec<String> {
    let mut candidates = normalized_candidates(member_name);

    if let Some((_, remainder)) = member_name.split_once(".dist-info/licenses/") {
        candidates.push(remainder.to_owned());
    }

    candidates.sort();
    candidates.dedup();
    candidates
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
