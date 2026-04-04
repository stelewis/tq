use std::path::{Path, PathBuf};

use tq_core::{RelativePathBuf, TargetName};
use tq_discovery::AnalysisIndex;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TargetContext {
    name: TargetName,
    package_path: RelativePathBuf,
    known_target_package_paths: Vec<RelativePathBuf>,
    test_root_display: RelativePathBuf,
}

impl TargetContext {
    #[must_use]
    pub const fn new(
        name: TargetName,
        package_path: RelativePathBuf,
        known_target_package_paths: Vec<RelativePathBuf>,
        test_root_display: RelativePathBuf,
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
    pub const fn test_root_display(&self) -> &RelativePathBuf {
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
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TargetPlanInput {
    name: TargetName,
    package_path: RelativePathBuf,
    source_package_root: PathBuf,
    test_root: PathBuf,
}

impl TargetPlanInput {
    #[must_use]
    pub fn new(
        name: TargetName,
        package_path: RelativePathBuf,
        source_package_root: impl Into<PathBuf>,
        test_root: impl Into<PathBuf>,
    ) -> Self {
        let source_package_root = source_package_root.into();
        let test_root = test_root.into();

        Self {
            name,
            package_path,
            source_package_root,
            test_root,
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
    pub fn source_package_root(&self) -> &Path {
        &self.source_package_root
    }

    #[must_use]
    pub fn test_root(&self) -> &Path {
        &self.test_root
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct PlannedTargetRun {
    target: TargetPlanInput,
    context: AnalysisContext,
}

impl PlannedTargetRun {
    pub(crate) const fn new(target: TargetPlanInput, context: AnalysisContext) -> Self {
        Self { target, context }
    }

    #[must_use]
    pub const fn target(&self) -> &TargetPlanInput {
        &self.target
    }

    #[must_use]
    pub const fn context(&self) -> &AnalysisContext {
        &self.context
    }
}

#[must_use]
pub fn path_to_forward_slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
