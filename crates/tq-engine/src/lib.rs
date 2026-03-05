use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use thiserror::Error;
use tq_discovery::{AnalysisIndex, DiscoveryError, build_analysis_index};

#[derive(Debug, Clone, Eq, PartialEq, Ord, PartialOrd)]
pub struct RuleId(String);

impl RuleId {
    pub fn parse(value: &str) -> Result<Self, EngineError> {
        if value.is_empty() {
            return Err(EngineError::Validation {
                message: "RuleId must be non-empty".to_owned(),
            });
        }

        let mut chars = value.chars();
        let Some(first) = chars.next() else {
            return Err(EngineError::Validation {
                message: "RuleId must be non-empty".to_owned(),
            });
        };

        if !first.is_ascii_lowercase() {
            return Err(EngineError::Validation {
                message: "RuleId must be kebab-case, e.g. mapping-missing-test".to_owned(),
            });
        }

        let mut previous_was_dash = false;
        for character in chars {
            if character == '-' {
                if previous_was_dash {
                    return Err(EngineError::Validation {
                        message: "RuleId must be kebab-case, e.g. mapping-missing-test".to_owned(),
                    });
                }
                previous_was_dash = true;
                continue;
            }

            if !character.is_ascii_lowercase() && !character.is_ascii_digit() {
                return Err(EngineError::Validation {
                    message: "RuleId must be kebab-case, e.g. mapping-missing-test".to_owned(),
                });
            }

            previous_was_dash = false;
        }

        if previous_was_dash {
            return Err(EngineError::Validation {
                message: "RuleId must be kebab-case, e.g. mapping-missing-test".to_owned(),
            });
        }

        Ok(Self(value.to_owned()))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl std::fmt::Display for RuleId {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        formatter.write_str(self.as_str())
    }
}

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

    fn with_target_if_missing(&self, target_name: &str) -> Self {
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
    #[must_use]
    pub const fn target(&self) -> &TargetPlanInput {
        &self.target
    }

    #[must_use]
    pub const fn context(&self) -> &AnalysisContext {
        &self.context
    }
}

pub fn plan_target_runs(
    configured_targets: &[TargetPlanInput],
    active_targets: &[TargetPlanInput],
) -> Result<Vec<PlannedTargetRun>, EngineError> {
    let known_target_package_paths = configured_targets
        .iter()
        .map(|target| path_to_forward_slashes(target.package_path()))
        .collect::<Vec<_>>();

    let mut planned_runs = Vec::with_capacity(active_targets.len());
    for target in active_targets {
        let index = build_analysis_index(target.source_package_root(), target.test_root())
            .map_err(EngineError::Discovery)?;

        let target_context = TargetContext::new(
            target.name(),
            path_to_forward_slashes(target.package_path()),
            known_target_package_paths.clone(),
            target
                .test_root()
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or_default(),
        )?;

        planned_runs.push(PlannedTargetRun {
            target: target.clone(),
            context: AnalysisContext::with_target(index, target_context),
        });
    }

    Ok(planned_runs)
}

pub trait Rule {
    fn rule_id(&self) -> &RuleId;
    fn evaluate(&self, context: &AnalysisContext) -> Vec<Finding>;
}

#[derive(Default)]
pub struct RuleEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleEngine {
    #[must_use]
    pub fn new(rules: Vec<Box<dyn Rule>>) -> Self {
        Self { rules }
    }

    #[must_use]
    pub fn run(&self, context: &AnalysisContext) -> EngineResult {
        let target_name = context.target().map(|target| target.name().to_owned());
        let mut findings = Vec::new();

        for rule in &self.rules {
            let _ = rule.rule_id();
            let rule_findings = rule.evaluate(context);
            match target_name.as_deref() {
                Some(target_name) => findings.extend(
                    rule_findings
                        .iter()
                        .map(|finding| finding.with_target_if_missing(target_name)),
                ),
                None => findings.extend(rule_findings),
            }
        }

        findings.sort_by_key(finding_sort_key);
        EngineResult::new(findings)
    }
}

#[must_use]
pub fn aggregate_results(results: &[EngineResult]) -> EngineResult {
    let mut findings = Vec::new();
    for result in results {
        findings.extend(result.findings().iter().cloned());
    }
    findings.sort_by_key(finding_sort_key);
    EngineResult::new(findings)
}

fn finding_sort_key(finding: &Finding) -> (String, String, u32, Severity, String, usize, String) {
    (
        finding.target().unwrap_or_default().to_owned(),
        path_to_forward_slashes(finding.path()),
        finding.line().unwrap_or(0),
        finding.severity(),
        finding.rule_id().as_str().to_owned(),
        finding.message().len(),
        finding.message().to_owned(),
    )
}

fn build_summary(findings: &[Finding]) -> FindingSummary {
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

fn path_to_forward_slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("{message}")]
    Validation { message: String },
    #[error(transparent)]
    Discovery(#[from] DiscoveryError),
}

#[must_use]
pub fn validate_unique_rule_ids(rule_ids: &[RuleId]) -> bool {
    let mut seen = BTreeSet::new();
    for rule_id in rule_ids {
        if !seen.insert(rule_id) {
            return false;
        }
    }
    true
}
