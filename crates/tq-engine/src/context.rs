use std::path::{Path, PathBuf};

use tq_discovery::AnalysisIndex;

use crate::EngineError;

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TargetContext {
    name: String,
    package_path: String,
    known_target_package_paths: Vec<String>,
    test_root_display: String,
}

impl TargetContext {
    pub fn new(
        name: impl Into<String>,
        package_path: impl Into<String>,
        known_target_package_paths: Vec<String>,
        test_root_display: impl Into<String>,
    ) -> Result<Self, EngineError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(EngineError::Validation {
                message: "Target context name must be non-empty".to_owned(),
            });
        }

        let package_path = package_path.into();
        if package_path.trim().is_empty() {
            return Err(EngineError::Validation {
                message: "Target context package path must be non-empty".to_owned(),
            });
        }

        let test_root_display = test_root_display.into();
        if test_root_display.trim().is_empty() {
            return Err(EngineError::Validation {
                message: "Target context test root display must be non-empty".to_owned(),
            });
        }

        Ok(Self {
            name,
            package_path,
            known_target_package_paths,
            test_root_display,
        })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn package_path(&self) -> &str {
        &self.package_path
    }

    #[must_use]
    pub fn known_target_package_paths(&self) -> &[String] {
        &self.known_target_package_paths
    }

    #[must_use]
    pub fn test_root_display(&self) -> &str {
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
    name: String,
    package_path: PathBuf,
    source_package_root: PathBuf,
    test_root: PathBuf,
}

impl TargetPlanInput {
    pub fn new(
        name: impl Into<String>,
        package_path: impl Into<PathBuf>,
        source_package_root: impl Into<PathBuf>,
        test_root: impl Into<PathBuf>,
    ) -> Result<Self, EngineError> {
        let name = name.into();
        if name.trim().is_empty() {
            return Err(EngineError::Validation {
                message: "Target name must be non-empty".to_owned(),
            });
        }

        let package_path = package_path.into();
        if package_path.as_os_str().is_empty() || package_path.is_absolute() {
            return Err(EngineError::Validation {
                message: "Target package path must be a non-empty relative path".to_owned(),
            });
        }

        let source_package_root = source_package_root.into();
        let test_root = test_root.into();

        Ok(Self {
            name,
            package_path,
            source_package_root,
            test_root,
        })
    }

    #[must_use]
    pub fn name(&self) -> &str {
        &self.name
    }

    #[must_use]
    pub fn package_path(&self) -> &Path {
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
