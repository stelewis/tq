use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use tq_core::{
    InitModulesMode, PackageName, QualifierStrategy, RelativePathBuf, RuleId, Severity, TargetName,
};

use crate::paths::normalize_absolute;

pub const DEFAULT_INIT_MODULES: InitModulesMode = InitModulesMode::Include;
pub const DEFAULT_MAX_TEST_FILE_NON_BLANK_LINES: u64 = 600;

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct PartialRuleConfig {
    pub init_modules: Option<InitModulesMode>,
    pub max_test_file_non_blank_lines: Option<u64>,
    pub qualifier_strategy: Option<QualifierStrategy>,
    pub allowed_qualifiers: Option<Vec<String>>,
    pub select: Option<Vec<RuleId>>,
    pub ignore: Option<Vec<RuleId>>,
    pub severity_overrides: Option<BTreeMap<RuleId, Severity>>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct PartialTargetConfig {
    pub name: Option<String>,
    pub package: Option<String>,
    pub source_root: Option<String>,
    pub test_root: Option<String>,
    pub init_modules: Option<InitModulesMode>,
    pub max_test_file_non_blank_lines: Option<u64>,
    pub qualifier_strategy: Option<QualifierStrategy>,
    pub allowed_qualifiers: Option<Vec<String>>,
    pub select: Option<Vec<RuleId>>,
    pub ignore: Option<Vec<RuleId>>,
    pub severity_overrides: Option<BTreeMap<RuleId, Severity>>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct PartialTqConfig {
    pub defaults: PartialRuleConfig,
    pub targets: Option<Vec<PartialTargetConfig>>,
    pub fail_on: Option<Severity>,
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct CliOverrides {
    init_modules: Option<InitModulesMode>,
    max_test_file_non_blank_lines: Option<u64>,
    qualifier_strategy: Option<QualifierStrategy>,
    allowed_qualifiers: Option<Vec<String>>,
    select: Option<Vec<RuleId>>,
    ignore: Option<Vec<RuleId>>,
    fail_on: Option<Severity>,
    severity_overrides: Option<BTreeMap<RuleId, Severity>>,
}

impl CliOverrides {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    #[must_use]
    pub const fn with_init_modules(mut self, init_modules: Option<InitModulesMode>) -> Self {
        self.init_modules = init_modules;
        self
    }

    #[must_use]
    pub const fn with_max_test_file_non_blank_lines(mut self, limit: Option<u64>) -> Self {
        self.max_test_file_non_blank_lines = limit;
        self
    }

    #[must_use]
    pub const fn with_qualifier_strategy(mut self, strategy: Option<QualifierStrategy>) -> Self {
        self.qualifier_strategy = strategy;
        self
    }

    #[must_use]
    pub fn with_allowed_qualifiers(mut self, allowed_qualifiers: Option<Vec<String>>) -> Self {
        self.allowed_qualifiers = allowed_qualifiers;
        self
    }

    #[must_use]
    pub fn with_select(mut self, select: Option<Vec<RuleId>>) -> Self {
        self.select = select;
        self
    }

    #[must_use]
    pub fn with_ignore(mut self, ignore: Option<Vec<RuleId>>) -> Self {
        self.ignore = ignore;
        self
    }

    #[must_use]
    pub const fn with_fail_on(mut self, fail_on: Option<Severity>) -> Self {
        self.fail_on = fail_on;
        self
    }

    #[must_use]
    pub fn with_severity_overrides(
        mut self,
        overrides: Option<BTreeMap<RuleId, Severity>>,
    ) -> Self {
        self.severity_overrides = overrides;
        self
    }

    pub(crate) const fn init_modules(&self) -> Option<InitModulesMode> {
        self.init_modules
    }

    pub(crate) const fn max_test_file_non_blank_lines(&self) -> Option<u64> {
        self.max_test_file_non_blank_lines
    }

    pub(crate) const fn qualifier_strategy(&self) -> Option<QualifierStrategy> {
        self.qualifier_strategy
    }

    pub(crate) fn allowed_qualifiers(&self) -> Option<&[String]> {
        self.allowed_qualifiers.as_deref()
    }

    pub(crate) fn clone_allowed_qualifiers(&self) -> Option<Vec<String>> {
        self.allowed_qualifiers.clone()
    }

    pub(crate) fn clone_select(&self) -> Option<Vec<RuleId>> {
        self.select.clone()
    }

    pub(crate) fn clone_ignore(&self) -> Option<Vec<RuleId>> {
        self.ignore.clone()
    }

    pub(crate) const fn fail_on(&self) -> Option<Severity> {
        self.fail_on
    }

    pub(crate) fn clone_severity_overrides(&self) -> Option<BTreeMap<RuleId, Severity>> {
        self.severity_overrides.clone()
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TqTargetConfig {
    pub(crate) name: TargetName,
    pub(crate) package: PackageName,
    pub(crate) source_root: PathBuf,
    pub(crate) test_root: PathBuf,
    pub(crate) test_root_display: PathBuf,
    pub(crate) init_modules: InitModulesMode,
    pub(crate) max_test_file_non_blank_lines: u64,
    pub(crate) qualifier_strategy: QualifierStrategy,
    pub(crate) allowed_qualifiers: Vec<String>,
    pub(crate) select: Vec<RuleId>,
    pub(crate) ignore: Vec<RuleId>,
    pub(crate) severity_overrides: BTreeMap<RuleId, Severity>,
}

impl TqTargetConfig {
    #[must_use]
    pub const fn name(&self) -> &TargetName {
        &self.name
    }

    #[must_use]
    pub const fn package(&self) -> &PackageName {
        &self.package
    }

    #[must_use]
    pub fn source_root(&self) -> &Path {
        &self.source_root
    }

    #[must_use]
    pub fn test_root(&self) -> &Path {
        &self.test_root
    }

    #[must_use]
    pub fn test_root_display(&self) -> &Path {
        &self.test_root_display
    }

    #[must_use]
    pub const fn init_modules(&self) -> InitModulesMode {
        self.init_modules
    }

    #[must_use]
    pub const fn max_test_file_non_blank_lines(&self) -> u64 {
        self.max_test_file_non_blank_lines
    }

    #[must_use]
    pub const fn qualifier_strategy(&self) -> QualifierStrategy {
        self.qualifier_strategy
    }

    #[must_use]
    pub fn allowed_qualifiers(&self) -> &[String] {
        &self.allowed_qualifiers
    }

    #[must_use]
    pub fn select(&self) -> &[RuleId] {
        &self.select
    }

    #[must_use]
    pub fn ignore(&self) -> &[RuleId] {
        &self.ignore
    }

    #[must_use]
    pub const fn severity_overrides(&self) -> &BTreeMap<RuleId, Severity> {
        &self.severity_overrides
    }

    #[must_use]
    pub const fn package_path(&self) -> &RelativePathBuf {
        self.package.relative_path()
    }

    #[must_use]
    pub fn source_package_root(&self) -> PathBuf {
        normalize_absolute(&self.source_root.join(self.package_path().as_path()))
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct TqConfig {
    pub(crate) targets: Vec<TqTargetConfig>,
    pub(crate) fail_on: Severity,
}

impl TqConfig {
    #[must_use]
    pub fn targets(&self) -> &[TqTargetConfig] {
        &self.targets
    }

    #[must_use]
    pub const fn fail_on(&self) -> Severity {
        self.fail_on
    }
}
