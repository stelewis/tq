use std::path::{Path, PathBuf};

use crate::EngineError;
use crate::{RuleId, TargetContext};

#[derive(Debug, Clone, Copy, Eq, PartialEq, Ord, PartialOrd)]
pub enum Severity {
    Error,
    Warning,
    Info,
}

impl Severity {
    #[must_use]
    pub const fn as_str(self) -> &'static str {
        match self {
            Self::Error => "error",
            Self::Warning => "warning",
            Self::Info => "info",
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Finding {
    rule_id: RuleId,
    severity: Severity,
    message: String,
    path: PathBuf,
    line: Option<u32>,
    suggestion: Option<String>,
    target: Option<String>,
}

impl Finding {
    pub fn new(
        rule_id: RuleId,
        severity: Severity,
        message: impl Into<String>,
        path: impl Into<PathBuf>,
        line: Option<u32>,
        suggestion: Option<String>,
        target: Option<String>,
    ) -> Result<Self, EngineError> {
        let message = message.into();
        if message.trim().is_empty() {
            return Err(EngineError::Validation {
                message: "Finding message must be non-empty".to_owned(),
            });
        }

        if line.is_some_and(|line| line < 1) {
            return Err(EngineError::Validation {
                message: "Finding line must be >= 1 when provided".to_owned(),
            });
        }

        if target.as_deref().is_some_and(|name| name.trim().is_empty()) {
            return Err(EngineError::Validation {
                message: "Finding target must be non-empty when provided".to_owned(),
            });
        }

        Ok(Self {
            rule_id,
            severity,
            message,
            path: path.into(),
            line,
            suggestion,
            target,
        })
    }

    #[must_use]
    pub const fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    #[must_use]
    pub const fn severity(&self) -> Severity {
        self.severity
    }

    #[must_use]
    pub fn message(&self) -> &str {
        &self.message
    }

    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    #[must_use]
    pub const fn line(&self) -> Option<u32> {
        self.line
    }

    #[must_use]
    pub fn suggestion(&self) -> Option<&str> {
        self.suggestion.as_deref()
    }

    #[must_use]
    pub fn target(&self) -> Option<&str> {
        self.target.as_deref()
    }

    #[must_use]
    pub(crate) fn with_target_if_missing(&self, target: Option<&TargetContext>) -> Self {
        let Some(target_name) = target.map(TargetContext::name) else {
            return self.clone();
        };

        if self.target.is_some() {
            return self.clone();
        }

        let mut cloned = self.clone();
        cloned.target = Some(target_name.to_owned());
        cloned
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct FindingSummary {
    errors: usize,
    warnings: usize,
    infos: usize,
}

impl FindingSummary {
    #[must_use]
    pub const fn new(errors: usize, warnings: usize, infos: usize) -> Self {
        Self {
            errors,
            warnings,
            infos,
        }
    }

    #[must_use]
    pub const fn errors(&self) -> usize {
        self.errors
    }

    #[must_use]
    pub const fn warnings(&self) -> usize {
        self.warnings
    }

    #[must_use]
    pub const fn infos(&self) -> usize {
        self.infos
    }

    #[must_use]
    pub const fn total(&self) -> usize {
        self.errors + self.warnings + self.infos
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct EngineResult {
    findings: Vec<Finding>,
    summary: FindingSummary,
}

impl EngineResult {
    #[must_use]
    pub fn new(findings: Vec<Finding>) -> Self {
        let summary = build_summary(&findings);
        Self { findings, summary }
    }

    #[must_use]
    pub fn findings(&self) -> &[Finding] {
        &self.findings
    }

    #[must_use]
    pub const fn summary(&self) -> &FindingSummary {
        &self.summary
    }

    #[must_use]
    pub const fn has_errors(&self) -> bool {
        self.summary.errors() > 0
    }
}

#[must_use]
pub fn build_summary(findings: &[Finding]) -> FindingSummary {
    let mut errors = 0;
    let mut warnings = 0;
    let mut infos = 0;

    for finding in findings {
        match finding.severity() {
            Severity::Error => errors += 1,
            Severity::Warning => warnings += 1,
            Severity::Info => infos += 1,
        }
    }

    FindingSummary::new(errors, warnings, infos)
}
