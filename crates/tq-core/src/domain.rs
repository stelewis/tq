use std::path::{Component, Path, PathBuf};

use thiserror::Error;

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct TargetName(String);

impl TargetName {
    pub fn parse(value: &str) -> Result<Self, TargetNameError> {
        validate_target_name(value)?;
        Ok(Self(value.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for TargetName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum TargetNameError {
    #[error("target name must be non-empty")]
    Empty,
    #[error("target name must be kebab-case")]
    InvalidFormat,
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct RelativePathBuf(PathBuf);

impl RelativePathBuf {
    pub fn new(path: impl Into<PathBuf>) -> Result<Self, RelativePathError> {
        let path = path.into();
        validate_relative_path(&path)?;
        Ok(Self(path))
    }

    #[must_use]
    pub fn as_path(&self) -> &Path {
        &self.0
    }
}

impl AsRef<Path> for RelativePathBuf {
    fn as_ref(&self) -> &Path {
        self.as_path()
    }
}

impl std::fmt::Display for RelativePathBuf {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(&self.0.to_string_lossy())
    }
}

impl From<RelativePathBuf> for PathBuf {
    fn from(value: RelativePathBuf) -> Self {
        value.0
    }
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum RelativePathError {
    #[error("relative path must be non-empty")]
    Empty,
    #[error("relative path must not contain platform path prefixes: {path}")]
    Prefix { path: PathBuf },
    #[error("relative path must not be absolute: {path}")]
    Absolute { path: PathBuf },
    #[error("relative path must not contain '.' components: {path}")]
    CurrentDir { path: PathBuf },
    #[error("relative path must not contain '..' components: {path}")]
    ParentDir { path: PathBuf },
}

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct PackageName {
    dotted: String,
    relative_path: RelativePathBuf,
}

impl PackageName {
    pub fn parse(value: &str) -> Result<Self, PackageNameError> {
        if value.is_empty() {
            return Err(PackageNameError::Empty);
        }

        let mut relative_path = PathBuf::new();
        for segment in value.split('.') {
            if !is_python_identifier(segment) {
                return Err(PackageNameError::InvalidSegment {
                    segment: segment.to_owned(),
                });
            }

            relative_path.push(segment);
        }

        let relative_path =
            RelativePathBuf::new(relative_path).map_err(PackageNameError::InvalidRelativePath)?;

        Ok(Self {
            dotted: value.to_owned(),
            relative_path,
        })
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.dotted
    }

    #[must_use]
    pub const fn relative_path(&self) -> &RelativePathBuf {
        &self.relative_path
    }
}

impl std::fmt::Display for PackageName {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum PackageNameError {
    #[error("package name must be non-empty")]
    Empty,
    #[error("package names must use dotted Python identifiers; invalid segment: {segment}")]
    InvalidSegment { segment: String },
    #[error(transparent)]
    InvalidRelativePath(#[from] RelativePathError),
}

fn validate_target_name(value: &str) -> Result<(), TargetNameError> {
    if value.is_empty() {
        return Err(TargetNameError::Empty);
    }

    let mut has_segment_char = false;
    let mut previous_was_dash = false;
    for character in value.chars() {
        if character == '-' {
            if !has_segment_char || previous_was_dash {
                return Err(TargetNameError::InvalidFormat);
            }
            previous_was_dash = true;
            continue;
        }

        if !character.is_ascii_lowercase() && !character.is_ascii_digit() {
            return Err(TargetNameError::InvalidFormat);
        }

        has_segment_char = true;
        previous_was_dash = false;
    }

    if previous_was_dash {
        return Err(TargetNameError::InvalidFormat);
    }

    Ok(())
}

fn validate_relative_path(path: &Path) -> Result<(), RelativePathError> {
    if path.as_os_str().is_empty() {
        return Err(RelativePathError::Empty);
    }
    if path.is_absolute() {
        return Err(RelativePathError::Absolute {
            path: path.to_path_buf(),
        });
    }

    for component in path.components() {
        match component {
            Component::Prefix(_) => {
                return Err(RelativePathError::Prefix {
                    path: path.to_path_buf(),
                });
            }
            Component::CurDir => {
                return Err(RelativePathError::CurrentDir {
                    path: path.to_path_buf(),
                });
            }
            Component::ParentDir => {
                return Err(RelativePathError::ParentDir {
                    path: path.to_path_buf(),
                });
            }
            Component::RootDir | Component::Normal(_) => {}
        }
    }

    Ok(())
}

fn is_python_identifier(value: &str) -> bool {
    let mut characters = value.chars();
    let Some(first) = characters.next() else {
        return false;
    };

    if !(first == '_' || first.is_ascii_alphabetic()) {
        return false;
    }

    characters.all(|character| character == '_' || character.is_ascii_alphanumeric())
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::{
        PackageName, PackageNameError, RelativePathBuf, RelativePathError, TargetName,
        TargetNameError,
    };

    #[test]
    fn target_name_accepts_kebab_case_values() {
        let name = TargetName::parse("python-core").expect("target name should parse");
        assert_eq!(name.as_str(), "python-core");
    }

    #[test]
    fn target_name_rejects_invalid_values() {
        let error = TargetName::parse("PythonCore").expect_err("target name should fail");
        assert_eq!(error, TargetNameError::InvalidFormat);
    }

    #[test]
    fn package_name_exposes_relative_path() {
        let package = PackageName::parse("tq.rules").expect("package should parse");
        assert_eq!(package.as_str(), "tq.rules");
        assert_eq!(package.relative_path().as_path(), Path::new("tq/rules"));
    }

    #[test]
    fn package_name_rejects_invalid_segments() {
        let error = PackageName::parse("tq.rules-").expect_err("package should fail");
        assert_eq!(
            error,
            PackageNameError::InvalidSegment {
                segment: "rules-".to_owned(),
            }
        );
    }

    #[test]
    fn relative_path_rejects_parent_components() {
        let error = RelativePathBuf::new("../tests").expect_err("path should fail");
        assert_eq!(
            error,
            RelativePathError::ParentDir {
                path: "../tests".into(),
            }
        );
    }

    #[test]
    fn relative_path_rejects_current_directory_components() {
        let error = RelativePathBuf::new("./tests").expect_err("path should fail");
        assert_eq!(
            error,
            RelativePathError::CurrentDir {
                path: "./tests".into(),
            }
        );
    }

    #[cfg(windows)]
    #[test]
    fn relative_path_rejects_platform_prefixes() {
        let prefixed = PathBuf::from("C:tests");

        let error = RelativePathBuf::new(prefixed.clone()).expect_err("path should fail");
        assert_eq!(error, RelativePathError::Prefix { path: prefixed });
    }
}
