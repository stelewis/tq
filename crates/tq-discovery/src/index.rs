use std::collections::BTreeSet;
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::DiscoveryError;

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

pub fn normalize_existing_dir(path: &Path) -> Result<PathBuf, DiscoveryError> {
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
        if path
            .components()
            .any(|component| matches!(component, Component::Prefix(_)))
        {
            return Err(DiscoveryError::PrefixedIndexPath { path });
        }

        if path.is_absolute() {
            return Err(DiscoveryError::AbsoluteIndexPath { path });
        }

        if path
            .components()
            .any(|component| matches!(component, Component::CurDir))
        {
            return Err(DiscoveryError::CurrentDirIndexPath { path });
        }

        if path
            .components()
            .any(|component| matches!(component, std::path::Component::ParentDir))
        {
            return Err(DiscoveryError::ParentDirIndexPath { path });
        }

        unique.insert(path);
    }

    Ok(unique.into_iter().collect())
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use crate::{AnalysisIndex, DiscoveryError};

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
    fn index_create_rejects_current_directory_components() {
        let temp = tempdir().expect("tempdir");
        let source_root = temp.path().join("src").join("tq");
        let test_root = temp.path().join("tests");
        std::fs::create_dir_all(&source_root).expect("create source root");
        std::fs::create_dir_all(&test_root).expect("create test root");

        let error = AnalysisIndex::create(
            &source_root,
            &test_root,
            vec![PathBuf::from("./a.py")],
            vec![PathBuf::from("tq/test_a.py")],
        )
        .expect_err("index should reject current-directory components");

        assert!(matches!(
            error,
            DiscoveryError::CurrentDirIndexPath { path } if path == std::path::Path::new("./a.py")
        ));
    }

    #[cfg(windows)]
    #[test]
    fn index_create_rejects_platform_prefixes() {
        let temp = tempdir().expect("tempdir");
        let source_root = temp.path().join("src").join("tq");
        let test_root = temp.path().join("tests");
        std::fs::create_dir_all(&source_root).expect("create source root");
        std::fs::create_dir_all(&test_root).expect("create test root");

        let prefixed = PathBuf::from("C:module.py");
        let error = AnalysisIndex::create(
            &source_root,
            &test_root,
            vec![prefixed.clone()],
            vec![PathBuf::from("tq/test_a.py")],
        )
        .expect_err("index should reject platform path prefixes");

        assert!(matches!(
            error,
            DiscoveryError::PrefixedIndexPath { path } if path == prefixed
        ));
    }
}
