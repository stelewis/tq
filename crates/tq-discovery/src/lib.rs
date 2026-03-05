use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AnalysisIndex {
    source_root: PathBuf,
    test_root: PathBuf,
    source_files: Vec<PathBuf>,
    test_files: Vec<PathBuf>,
}

impl AnalysisIndex {
    pub fn create(
        source_root: &Path,
        test_root: &Path,
        source_files: impl IntoIterator<Item = PathBuf>,
        test_files: impl IntoIterator<Item = PathBuf>,
    ) -> Result<Self, DiscoveryError> {
        let normalized_source_root = normalize_existing_dir(source_root)?;
        let normalized_test_root = normalize_existing_dir(test_root)?;

        let source_files = normalize_relative_paths(source_files)?;
        let test_files = normalize_relative_paths(test_files)?;

        Ok(Self {
            source_root: normalized_source_root,
            test_root: normalized_test_root,
            source_files,
            test_files,
        })
    }

    #[must_use]
    pub fn source_root(&self) -> &Path {
        &self.source_root
    }

    #[must_use]
    pub fn test_root(&self) -> &Path {
        &self.test_root
    }

    #[must_use]
    pub fn source_files(&self) -> &[PathBuf] {
        &self.source_files
    }

    #[must_use]
    pub fn test_files(&self) -> &[PathBuf] {
        &self.test_files
    }
}

#[derive(Debug, Error)]
pub enum DiscoveryError {
    #[error("{path} must be an existing directory")]
    NotDirectory { path: PathBuf },
    #[error("{operation} failed for {path}: {source}")]
    Io {
        operation: &'static str,
        path: PathBuf,
        source: std::io::Error,
    },
    #[error("{message}")]
    Validation { message: String },
}

pub fn build_analysis_index(
    source_root: &Path,
    test_root: &Path,
) -> Result<AnalysisIndex, DiscoveryError> {
    let source_root = normalize_existing_dir(source_root)?;
    let test_root = normalize_existing_dir(test_root)?;

    let source_files = scan_files(&source_root, is_source_module)?;
    let test_files = scan_files(&test_root, is_test_module)?;

    AnalysisIndex::create(&source_root, &test_root, source_files, test_files)
}

fn normalize_existing_dir(path: &Path) -> Result<PathBuf, DiscoveryError> {
    let normalized = fs::canonicalize(path).map_err(|source| DiscoveryError::Io {
        operation: "canonicalize",
        path: path.to_path_buf(),
        source,
    })?;

    if normalized.is_dir() {
        Ok(normalized)
    } else {
        Err(DiscoveryError::NotDirectory { path: normalized })
    }
}

fn normalize_relative_paths(
    paths: impl IntoIterator<Item = PathBuf>,
) -> Result<Vec<PathBuf>, DiscoveryError> {
    let mut unique = BTreeSet::new();

    for path in paths {
        if path.is_absolute() {
            return Err(DiscoveryError::Validation {
                message: format!("index file paths must be relative: {}", path.display()),
            });
        }

        if path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(DiscoveryError::Validation {
                message: format!("index file paths must not contain '..': {}", path.display()),
            });
        }

        unique.insert(path);
    }

    Ok(unique.into_iter().collect())
}

fn scan_files(root: &Path, matcher: fn(&Path) -> bool) -> Result<Vec<PathBuf>, DiscoveryError> {
    let mut discovered = Vec::new();
    scan_recursive(root, root, matcher, &mut discovered)?;
    Ok(discovered)
}

fn scan_recursive(
    root: &Path,
    directory: &Path,
    matcher: fn(&Path) -> bool,
    discovered: &mut Vec<PathBuf>,
) -> Result<(), DiscoveryError> {
    let entries = fs::read_dir(directory).map_err(|source| DiscoveryError::Io {
        operation: "read_dir",
        path: directory.to_path_buf(),
        source,
    })?;

    for entry in entries {
        let entry = entry.map_err(|source| DiscoveryError::Io {
            operation: "read_dir_entry",
            path: directory.to_path_buf(),
            source,
        })?;

        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| DiscoveryError::Io {
            operation: "read_file_type",
            path: path.clone(),
            source,
        })?;

        if is_ignored_path(&path) {
            continue;
        }

        if file_type.is_dir() {
            scan_recursive(root, &path, matcher, discovered)?;
            continue;
        }

        if file_type.is_file() && matcher(&path) {
            let relative = path
                .strip_prefix(root)
                .map_err(|_| DiscoveryError::Validation {
                    message: format!(
                        "discovered file {} is not under root {}",
                        path.display(),
                        root.display()
                    ),
                })?;
            discovered.push(relative.to_path_buf());
        }
    }

    Ok(())
}

fn is_source_module(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "py")
}

fn is_test_module(path: &Path) -> bool {
    let is_python = path.extension().is_some_and(|extension| extension == "py");
    let is_test_name = path
        .file_name()
        .and_then(std::ffi::OsStr::to_str)
        .is_some_and(|name| name.starts_with("test_"));
    is_python && is_test_name
}

fn is_ignored_path(path: &Path) -> bool {
    path.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|segment| segment == "__pycache__")
    })
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use tempfile::tempdir;

    use crate::{AnalysisIndex, build_analysis_index};

    fn write(path: &Path) {
        std::fs::create_dir_all(path.parent().expect("file parent path must exist"))
            .expect("create parent directories");
        std::fs::write(path, "pass\n").expect("write fixture file");
    }

    #[test]
    fn index_create_sorts_and_deduplicates_paths() {
        let temp = tempdir().expect("tempdir");
        let source_root = temp.path().join("src").join("tq");
        let test_root = temp.path().join("tests");
        std::fs::create_dir_all(&source_root).expect("create source root");
        std::fs::create_dir_all(&test_root).expect("create test root");

        let index = AnalysisIndex::create(
            &source_root,
            &test_root,
            vec![
                PathBuf::from("z.py"),
                PathBuf::from("a.py"),
                PathBuf::from("a.py"),
            ],
            vec![
                PathBuf::from("tq/test_z.py"),
                PathBuf::from("tq/test_a.py"),
                PathBuf::from("tq/test_a.py"),
            ],
        )
        .expect("index should be created");

        assert_eq!(
            index.source_files(),
            &[PathBuf::from("a.py"), PathBuf::from("z.py")]
        );
        assert_eq!(
            index.test_files(),
            &[PathBuf::from("tq/test_a.py"), PathBuf::from("tq/test_z.py")]
        );
    }

    #[test]
    fn build_analysis_index_discovers_expected_files() {
        let temp = tempdir().expect("tempdir");
        let source_root = temp.path().join("src").join("tq");
        let test_root = temp.path().join("tests");

        write(&source_root.join("engine").join("runner.py"));
        write(&test_root.join("tq").join("engine").join("test_runner.py"));

        let index = build_analysis_index(&source_root, &test_root).expect("index should build");

        assert_eq!(index.source_files(), &[PathBuf::from("engine/runner.py")]);
        assert_eq!(
            index.test_files(),
            &[PathBuf::from("tq/engine/test_runner.py")]
        );
    }

    #[test]
    fn build_analysis_index_ignores_pycache_entries() {
        let temp = tempdir().expect("tempdir");
        let source_root = temp.path().join("src").join("tq");
        let test_root = temp.path().join("tests");

        write(&source_root.join("__pycache__").join("cached.py"));
        write(&source_root.join("engine").join("runner.py"));
        write(&test_root.join("__pycache__").join("test_cached.py"));
        write(&test_root.join("tq").join("engine").join("test_runner.py"));

        let index = build_analysis_index(&source_root, &test_root).expect("index should build");

        assert_eq!(index.source_files(), &[PathBuf::from("engine/runner.py")]);
        assert_eq!(
            index.test_files(),
            &[PathBuf::from("tq/engine/test_runner.py")]
        );
    }
}
