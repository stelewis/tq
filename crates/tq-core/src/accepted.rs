use std::num::NonZeroU64;
use std::path::Path;

use thiserror::Error;

use crate::{RelativePathBuf, RelativePathError, RuleId, is_python_module, is_python_test_file};

#[derive(Debug, Clone, Copy, Eq, PartialEq)]
pub enum AcceptedFindingKind {
    MappingMissingTest,
    TestFileTooLarge { max_non_blank_lines: NonZeroU64 },
}

impl AcceptedFindingKind {
    #[must_use]
    pub const fn rule_id(self) -> RuleId {
        match self {
            Self::MappingMissingTest => RuleId::from_static("mapping-missing-test"),
            Self::TestFileTooLarge { .. } => RuleId::from_static("test-file-too-large"),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct AcceptedFinding {
    kind: AcceptedFindingKind,
    path: RelativePathBuf,
    rationale: String,
}

impl AcceptedFinding {
    pub fn new(
        kind: AcceptedFindingKind,
        path: &str,
        rationale: &str,
    ) -> Result<Self, AcceptedFindingError> {
        if path.contains(['\\', ':', '*', '?', '[', ']', '{', '}'])
            || path
                .split('/')
                .any(|part| part.is_empty() || part == "." || part == "..")
        {
            return Err(AcceptedFindingError::NonCanonicalPath);
        }
        let path = RelativePathBuf::new(path)?;
        let valid_file = match kind {
            AcceptedFindingKind::MappingMissingTest => is_python_module(path.as_path()),
            AcceptedFindingKind::TestFileTooLarge { .. } => is_python_test_file(path.as_path()),
        };
        if !valid_file {
            return Err(AcceptedFindingError::InvalidFile);
        }
        if rationale.trim().is_empty() {
            return Err(AcceptedFindingError::EmptyRationale);
        }
        Ok(Self {
            kind,
            path,
            rationale: rationale.trim().to_owned(),
        })
    }

    #[must_use]
    pub const fn kind(&self) -> AcceptedFindingKind {
        self.kind
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        self.path.as_path()
    }

    #[must_use]
    pub fn rationale(&self) -> &str {
        &self.rationale
    }
}

#[derive(Debug, Error, Clone, Eq, PartialEq)]
pub enum AcceptedFindingError {
    #[error(
        "path must be an exact relative path with '/' separators, without empty, '.', '..', prefix or glob components"
    )]
    NonCanonicalPath,
    #[error(transparent)]
    RelativePath(#[from] RelativePathError),
    #[error(
        "path must name a Python module for mapping-missing-test or a test_*.py file for test-file-too-large"
    )]
    InvalidFile,
    #[error("rationale must be non-empty")]
    EmptyRationale,
}
