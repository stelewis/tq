use std::path::{Path, PathBuf};

use tempfile::TempDir;
use tq_discovery::AnalysisIndex;
use tq_engine::{AnalysisContext, TargetContext};

pub fn fixture_workspace() -> TempDir {
    tempfile::tempdir().expect("tempdir")
}

pub fn create_dirs(root: &Path) -> (PathBuf, PathBuf) {
    let source_root = root.join("src").join("tq");
    let test_root = root.join("tests");
    std::fs::create_dir_all(&source_root).expect("create source root");
    std::fs::create_dir_all(&test_root).expect("create test root");
    (source_root, test_root)
}

pub fn context_with_target(
    source_root: &Path,
    test_root: &Path,
    source_files: Vec<PathBuf>,
    test_files: Vec<PathBuf>,
    package_path: &str,
    known_target_package_paths: Vec<String>,
) -> AnalysisContext {
    let index = AnalysisIndex::create(source_root, test_root, source_files, test_files)
        .expect("index should be created");
    let target = TargetContext::new(
        "active",
        package_path,
        known_target_package_paths,
        test_root
            .file_name()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or("tests"),
    )
    .expect("target context should be valid");

    AnalysisContext::with_target(index, target)
}
