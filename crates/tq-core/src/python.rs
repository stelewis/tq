use std::num::NonZeroU64;
use std::path::{Path, PathBuf};

pub const DEFAULT_MAX_TEST_FILE_NON_BLANK_LINES: NonZeroU64 =
    NonZeroU64::new(600).expect("default test file line limit is a non-zero literal");

#[must_use]
pub fn is_python_module(path: &Path) -> bool {
    path.extension().is_some_and(|extension| extension == "py")
}

#[must_use]
pub fn python_module_name(path: &Path) -> Option<&str> {
    is_python_module(path)
        .then(|| path.file_stem().and_then(std::ffi::OsStr::to_str))
        .flatten()
}

#[must_use]
pub fn python_test_module_name(path: &Path) -> Option<&str> {
    python_module_name(path)?.strip_prefix("test_")
}

#[must_use]
pub fn is_python_test_file(path: &Path) -> bool {
    python_test_module_name(path).is_some_and(|module_name| !module_name.is_empty())
}

#[must_use]
pub fn unit_test_path_for_source(source_file: &Path, package_path: &Path) -> Option<PathBuf> {
    let module_name = python_module_name(source_file)?;
    let test_file_name = format!("test_{module_name}.py");
    Some(
        package_path
            .join(source_file.parent().unwrap_or_else(|| Path::new("")))
            .join(test_file_name),
    )
}

#[must_use]
pub fn source_directory_for_unit_test(test_file: &Path, package_path: &Path) -> Option<PathBuf> {
    test_file
        .strip_prefix(package_path)
        .ok()?
        .parent()
        .map(Path::to_path_buf)
}

#[cfg(test)]
mod tests {
    use std::path::{Path, PathBuf};

    use super::{
        is_python_module, is_python_test_file, python_test_module_name,
        source_directory_for_unit_test, unit_test_path_for_source,
    };

    #[test]
    fn python_vocabulary_is_case_sensitive_and_requires_canonical_test_names() {
        assert!(is_python_module(Path::new("module.py")));
        assert!(!is_python_module(Path::new("module.PY")));
        assert!(is_python_test_file(Path::new("test_module.py")));
        assert!(!is_python_test_file(Path::new("module_test.py")));
        assert!(!is_python_test_file(Path::new("test_.py")));
        assert!(!is_python_test_file(Path::new("test_module.PY")));
        assert_eq!(
            python_test_module_name(Path::new("test_module_regression.py")),
            Some("module_regression")
        );
    }

    #[test]
    fn unit_test_path_preserves_source_layout_under_package() {
        assert_eq!(
            unit_test_path_for_source(Path::new("engine/runner.py"), Path::new("tq")),
            Some(PathBuf::from("tq/engine/test_runner.py"))
        );
        assert_eq!(
            unit_test_path_for_source(Path::new("__init__.py"), Path::new("tq")),
            Some(PathBuf::from("tq/test___init__.py"))
        );
    }

    #[test]
    fn unit_test_source_directory_requires_package_prefix() {
        assert_eq!(
            source_directory_for_unit_test(Path::new("tq/engine/test_runner.py"), Path::new("tq")),
            Some(PathBuf::from("engine"))
        );
        assert_eq!(
            source_directory_for_unit_test(Path::new("other/test_runner.py"), Path::new("tq")),
            None
        );
    }
}
