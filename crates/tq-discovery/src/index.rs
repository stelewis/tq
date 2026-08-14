use std::collections::{BTreeMap, BTreeSet, btree_map::Entry};
use std::fs;
use std::path::{Component, Path, PathBuf};

use crate::DiscoveryError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AnalyzedTestFile {
    path: PathBuf,
    non_blank_non_comment_lines: u64,
}

impl AnalyzedTestFile {
    #[must_use]
    pub const fn new(path: PathBuf, non_blank_non_comment_lines: u64) -> Self {
        Self {
            path,
            non_blank_non_comment_lines,
        }
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub const fn non_blank_non_comment_lines(&self) -> u64 {
        self.non_blank_non_comment_lines
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AnalysisIndex {
    source_root: PathBuf,
    test_root: PathBuf,
    source_files: Vec<PathBuf>,
    test_files: Vec<AnalyzedTestFile>,
}

impl AnalysisIndex {
    pub fn create(
        source_root: &Path,
        test_root: &Path,
        source_files: impl IntoIterator<Item = PathBuf>,
        test_files: impl IntoIterator<Item = AnalyzedTestFile>,
    ) -> Result<Self, DiscoveryError> {
        let normalized_source_root = normalize_existing_dir(source_root)?;
        let normalized_test_root = normalize_existing_dir(test_root)?;
        Self::from_normalized_roots(
            normalized_source_root,
            normalized_test_root,
            source_files,
            test_files,
        )
    }

    pub(crate) fn from_normalized_roots(
        source_root: PathBuf,
        test_root: PathBuf,
        source_files: impl IntoIterator<Item = PathBuf>,
        test_files: impl IntoIterator<Item = AnalyzedTestFile>,
    ) -> Result<Self, DiscoveryError> {
        let source_files = normalize_relative_paths(source_files)?;
        let test_files = normalize_test_files(test_files)?;

        Ok(Self {
            source_root,
            test_root,
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
    pub fn test_files(&self) -> &[AnalyzedTestFile] {
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

fn normalize_test_files(
    files: impl IntoIterator<Item = AnalyzedTestFile>,
) -> Result<Vec<AnalyzedTestFile>, DiscoveryError> {
    let mut by_path = BTreeMap::new();
    for file in files {
        let path = normalize_relative_path(file.path)?;
        match by_path.entry(path.clone()) {
            Entry::Vacant(entry) => {
                entry.insert(AnalyzedTestFile::new(
                    path,
                    file.non_blank_non_comment_lines,
                ));
            }
            Entry::Occupied(entry)
                if entry.get().non_blank_non_comment_lines != file.non_blank_non_comment_lines =>
            {
                return Err(DiscoveryError::ConflictingTestFileAnalysis { path });
            }
            Entry::Occupied(_) => {}
        }
    }
    Ok(by_path.into_values().collect())
}

fn normalize_relative_path(path: PathBuf) -> Result<PathBuf, DiscoveryError> {
    normalize_relative_paths([path]).map(|mut paths| paths.remove(0))
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use tempfile::tempdir;

    use super::AnalyzedTestFile;
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
                AnalyzedTestFile::new(PathBuf::from("tq/test_z.py"), 1),
                AnalyzedTestFile::new(PathBuf::from("tq/test_a.py"), 1),
                AnalyzedTestFile::new(PathBuf::from("tq/test_a.py"), 1),
            ],
        )
        .expect("index should be created");

        assert_eq!(
            index.source_files(),
            &[PathBuf::from("a.py"), PathBuf::from("z.py")]
        );
        assert_eq!(
            index.test_files(),
            &[
                AnalyzedTestFile::new(PathBuf::from("tq/test_a.py"), 1),
                AnalyzedTestFile::new(PathBuf::from("tq/test_z.py"), 1),
            ]
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
            vec![AnalyzedTestFile::new(PathBuf::from("tq/test_a.py"), 1)],
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
            vec![AnalyzedTestFile::new(PathBuf::from("tq/test_a.py"), 1)],
        )
        .expect_err("index should reject platform path prefixes");

        assert!(matches!(
            error,
            DiscoveryError::PrefixedIndexPath { path } if path == prefixed
        ));
    }

    #[test]
    fn index_create_rejects_conflicting_test_file_analysis() {
        let temp = tempdir().expect("tempdir");
        let source_root = temp.path().join("src").join("tq");
        let test_root = temp.path().join("tests");
        std::fs::create_dir_all(&source_root).expect("create source root");
        std::fs::create_dir_all(&test_root).expect("create test root");
        let path = PathBuf::from("tq/test_a.py");

        let error = AnalysisIndex::create(
            &source_root,
            &test_root,
            vec![PathBuf::from("a.py")],
            vec![
                AnalyzedTestFile::new(path.clone(), 1),
                AnalyzedTestFile::new(path.clone(), 2),
            ],
        )
        .expect_err("conflicting analysis must fail");

        assert!(matches!(
            error,
            DiscoveryError::ConflictingTestFileAnalysis { path: error_path }
                if error_path == path
        ));
    }
}
