use std::fs;
use std::path::{Path, PathBuf};

use crate::index::normalize_existing_dir;
use crate::{AnalysisIndex, DiscoveryError};

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

    use crate::build_analysis_index;

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
