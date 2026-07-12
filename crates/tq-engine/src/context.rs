use std::path::{Path, PathBuf};

use tq_core::{RelativePathBuf, TargetName};
use tq_discovery::AnalysisIndex;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TargetContext {
    name: TargetName,
    package_path: RelativePathBuf,
    known_target_package_paths: Vec<RelativePathBuf>,
    test_root_display: PathBuf,
}

impl TargetContext {
    #[must_use]
    pub const fn new(
        name: TargetName,
        package_path: RelativePathBuf,
        known_target_package_paths: Vec<RelativePathBuf>,
        test_root_display: PathBuf,
    ) -> Self {
        Self {
            name,
            package_path,
            known_target_package_paths,
            test_root_display,
        }
    }

    #[must_use]
    pub const fn name(&self) -> &TargetName {
        &self.name
    }

    #[must_use]
    pub const fn package_path(&self) -> &RelativePathBuf {
        &self.package_path
    }

    #[must_use]
    pub fn known_target_package_paths(&self) -> &[RelativePathBuf] {
        &self.known_target_package_paths
    }

    #[must_use]
    pub fn test_root_display(&self) -> &Path {
        &self.test_root_display
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AnalysisContext {
    index: AnalysisIndex,
    target: Option<TargetContext>,
}

impl AnalysisContext {
    #[must_use]
    pub const fn new(index: AnalysisIndex) -> Self {
        Self {
            index,
            target: None,
        }
    }

    #[must_use]
    pub const fn with_target(index: AnalysisIndex, target: TargetContext) -> Self {
        Self {
            index,
            target: Some(target),
        }
    }

    #[must_use]
    pub const fn index(&self) -> &AnalysisIndex {
        &self.index
    }

    #[must_use]
    pub const fn target(&self) -> Option<&TargetContext> {
        self.target.as_ref()
    }

    #[must_use]
    pub fn package_path(&self) -> &Path {
        self.target().map_or_else(
            || {
                self.index()
                    .source_root()
                    .file_name()
                    .map_or_else(|| Path::new(""), Path::new)
            },
            |target| target.package_path().as_path(),
        )
    }

    #[must_use]
    pub fn test_root_display(&self) -> &Path {
        self.target().map_or_else(
            || {
                self.index()
                    .test_root()
                    .file_name()
                    .map_or_else(|| Path::new(""), Path::new)
            },
            TargetContext::test_root_display,
        )
    }

    #[must_use]
    pub fn known_target_package_paths(&self) -> &[RelativePathBuf] {
        self.target()
            .map_or_else(|| &[], TargetContext::known_target_package_paths)
    }
}
