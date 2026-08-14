use std::fs::{self, File};
use std::io::{BufRead, BufReader};
use std::path::{Path, PathBuf};

use crate::index::normalize_existing_dir;
use crate::{AnalysisIndex, AnalyzedTestFile, DiscoveryError};
use tq_core::{is_python_module, is_python_test_file};

pub fn build_analysis_index(
    source_root: &Path,
    test_root: &Path,
) -> Result<AnalysisIndex, DiscoveryError> {
    let source_root = normalize_existing_dir(source_root)?;
    let test_root = normalize_existing_dir(test_root)?;

    let source_files = scan_files(&source_root, is_python_module)?;
    let test_files = scan_files(&test_root, is_python_test_file)?
        .into_iter()
        .map(|path| analyze_test_file(&test_root, path))
        .collect::<Result<Vec<_>, _>>()?;

    AnalysisIndex::from_normalized_roots(source_root, test_root, source_files, test_files)
}

fn analyze_test_file(root: &Path, path: PathBuf) -> Result<AnalyzedTestFile, DiscoveryError> {
    let full_path = root.join(&path);
    let file = File::open(&full_path).map_err(|source| DiscoveryError::Io {
        operation: "read_test_file",
        path: full_path.clone(),
        source,
    })?;
    let mut non_blank_non_comment_lines = 0;
    for line in BufReader::new(file).lines() {
        let line = line.map_err(|source| DiscoveryError::Io {
            operation: "read_test_file",
            path: full_path.clone(),
            source,
        })?;
        let stripped = line.trim();
        if !stripped.is_empty() && !stripped.starts_with('#') {
            non_blank_non_comment_lines += 1;
        }
    }

    Ok(AnalyzedTestFile::new(path, non_blank_non_comment_lines))
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

        if file_type.is_symlink() {
            continue;
        }

        if is_ignored_path(&path) {
            continue;
        }

        if file_type.is_dir() {
            scan_recursive(root, &path, matcher, discovered)?;
            continue;
        }

        if file_type.is_file() && matcher(&path) {
            let relative =
                path.strip_prefix(root)
                    .map_err(|_| DiscoveryError::DiscoveredPathOutsideRoot {
                        path: path.clone(),
                        root: root.to_path_buf(),
                    })?;
            discovered.push(relative.to_path_buf());
        }
    }

    Ok(())
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
    fn build_analysis_index_discovers_expected_files() {
        let temp = tempdir().expect("tempdir");
        let source_root = temp.path().join("src").join("tq");
        let test_root = temp.path().join("tests");

        write(&source_root.join("engine").join("runner.py"));
        write(&test_root.join("tq").join("engine").join("test_runner.py"));

        let index = build_analysis_index(&source_root, &test_root).expect("index should build");

        assert_eq!(index.source_files(), &[PathBuf::from("engine/runner.py")]);
        assert_eq!(
            test_paths(&index),
            &[PathBuf::from("tq/engine/test_runner.py")]
        );
        assert_eq!(index.test_files()[0].non_blank_non_comment_lines(), 1);
    }

    #[test]
    fn build_analysis_index_fails_when_test_file_cannot_be_decoded() {
        let temp = tempdir().expect("tempdir");
        let source_root = temp.path().join("src").join("tq");
        let test_root = temp.path().join("tests");
        write(&source_root.join("module.py"));
        let test_file = test_root.join("tq").join("test_module.py");
        std::fs::create_dir_all(test_file.parent().expect("test file parent"))
            .expect("create test package dir");
        std::fs::write(&test_file, [0xff, 0xfe, 0xfa]).expect("write invalid UTF-8");
        let canonical_test_file =
            std::fs::canonicalize(&test_file).expect("canonicalize test fixture");

        let error = build_analysis_index(&source_root, &test_root)
            .expect_err("invalid UTF-8 must fail discovery");

        assert!(matches!(
            error,
            crate::DiscoveryError::Io {
                operation: "read_test_file",
                path,
                ..
            } if path == canonical_test_file
        ));
    }

    #[cfg(unix)]
    #[test]
    fn build_analysis_index_ignores_symlink_entries() {
        use std::os::unix::fs::symlink;

        let temp = tempdir().expect("tempdir");
        let source_root = temp.path().join("src").join("tq");
        let test_root = temp.path().join("tests");
        write(&source_root.join("module.py"));
        std::fs::create_dir_all(test_root.join("tq")).expect("create test package dir");
        let external_test = temp.path().join("external").join("test_module.py");
        write(&external_test);
        symlink(&external_test, test_root.join("tq").join("test_module.py"))
            .expect("create test symlink");

        let index = build_analysis_index(&source_root, &test_root).expect("index should build");

        assert!(index.test_files().is_empty());
    }

    #[test]
    fn build_analysis_index_ignores_non_canonical_python_extensions() {
        let temp = tempdir().expect("tempdir");
        let source_root = temp.path().join("src").join("tq");
        let test_root = temp.path().join("tests");

        write(&source_root.join("ignored.PY"));
        write(&source_root.join("module.py"));
        write(&test_root.join("tq").join("test_ignored.PY"));
        write(&test_root.join("tq").join("test_module.py"));

        let index = build_analysis_index(&source_root, &test_root).expect("index should build");

        assert_eq!(index.source_files(), &[PathBuf::from("module.py")]);
        assert_eq!(test_paths(&index), &[PathBuf::from("tq/test_module.py")]);
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
            test_paths(&index),
            &[PathBuf::from("tq/engine/test_runner.py")]
        );
    }

    fn test_paths(index: &AnalysisIndex) -> Vec<PathBuf> {
        index
            .test_files()
            .iter()
            .map(|file| file.path().to_path_buf())
            .collect()
    }
}
