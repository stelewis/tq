use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tq_core::{QualifierStrategy, RuleId};
use tq_engine::{AnalysisContext, Rule};

use crate::error::RulesError;
use crate::file_too_large::TestFileTooLargeRule;
use crate::mapping_missing_test::MappingMissingTestRule;
use crate::orphaned_test::OrphanedTestRule;
use crate::structure_mismatch::StructureMismatchRule;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum BuiltinRule {
    MappingMissingTest,
    StructureMismatch,
    TestFileTooLarge,
    OrphanedTest,
}

impl BuiltinRule {
    const ALL: [Self; 4] = [
        Self::MappingMissingTest,
        Self::StructureMismatch,
        Self::TestFileTooLarge,
        Self::OrphanedTest,
    ];

    #[must_use]
    const fn as_str(self) -> &'static str {
        match self {
            Self::MappingMissingTest => "mapping-missing-test",
            Self::StructureMismatch => "structure-mismatch",
            Self::TestFileTooLarge => "test-file-too-large",
            Self::OrphanedTest => "orphaned-test",
        }
    }

    fn rule_id(self) -> Result<RuleId, RulesError> {
        parse_builtin_rule_id(self.as_str())
    }

    #[must_use]
    fn from_rule_id(rule_id: &RuleId) -> Option<Self> {
        match rule_id.as_str() {
            "mapping-missing-test" => Some(Self::MappingMissingTest),
            "structure-mismatch" => Some(Self::StructureMismatch),
            "test-file-too-large" => Some(Self::TestFileTooLarge),
            "orphaned-test" => Some(Self::OrphanedTest),
            _ => None,
        }
    }

    fn build(self, options: &BuiltinRuleOptions) -> Result<Box<dyn Rule>, RulesError> {
        match self {
            Self::MappingMissingTest => Ok(Box::new(MappingMissingTestRule::new(
                options.ignore_init_modules(),
                options.qualifier_strategy(),
                options.allowed_qualifiers().clone(),
            )?)),
            Self::StructureMismatch => Ok(Box::new(StructureMismatchRule::new()?)),
            Self::TestFileTooLarge => Ok(Box::new(TestFileTooLargeRule::new(
                options.max_test_file_non_blank_lines(),
            )?)),
            Self::OrphanedTest => Ok(Box::new(OrphanedTestRule::new(
                options.qualifier_strategy(),
                options.allowed_qualifiers().clone(),
            )?)),
        }
    }
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct BuiltinRuleOptions {
    ignore_init_modules: bool,
    max_test_file_non_blank_lines: u64,
    qualifier_strategy: QualifierStrategy,
    allowed_qualifiers: BTreeSet<String>,
}

impl BuiltinRuleOptions {
    pub fn new(
        ignore_init_modules: bool,
        max_test_file_non_blank_lines: u64,
        qualifier_strategy: QualifierStrategy,
        allowed_qualifiers: impl IntoIterator<Item = String>,
    ) -> Result<Self, RulesError> {
        if max_test_file_non_blank_lines < 1 {
            return Err(RulesError::validation(
                "max_test_file_non_blank_lines must be >= 1",
            ));
        }

        let allowed_qualifiers = normalize_non_empty_trimmed_strings(allowed_qualifiers);
        if qualifier_strategy == QualifierStrategy::Allowlist && allowed_qualifiers.is_empty() {
            return Err(RulesError::validation(
                "allowed_qualifiers must be non-empty for allowlist strategy",
            ));
        }

        Ok(Self {
            ignore_init_modules,
            max_test_file_non_blank_lines,
            qualifier_strategy,
            allowed_qualifiers,
        })
    }

    #[must_use]
    pub const fn ignore_init_modules(&self) -> bool {
        self.ignore_init_modules
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
    pub const fn allowed_qualifiers(&self) -> &BTreeSet<String> {
        &self.allowed_qualifiers
    }
}

impl Default for BuiltinRuleOptions {
    fn default() -> Self {
        Self {
            ignore_init_modules: false,
            max_test_file_non_blank_lines: 600,
            qualifier_strategy: QualifierStrategy::AnySuffix,
            allowed_qualifiers: BTreeSet::new(),
        }
    }
}

#[derive(Debug, Clone, Default, Eq, PartialEq)]
pub struct RuleSelection {
    select: Vec<RuleId>,
    ignore: Vec<RuleId>,
}

impl RuleSelection {
    #[must_use]
    pub const fn new(select: Vec<RuleId>, ignore: Vec<RuleId>) -> Self {
        Self { select, ignore }
    }

    #[must_use]
    pub fn select(&self) -> &[RuleId] {
        &self.select
    }

    #[must_use]
    pub fn ignore(&self) -> &[RuleId] {
        &self.ignore
    }
}

pub struct BuiltinRuleRegistry;

impl BuiltinRuleRegistry {
    pub fn build_rules(
        selection: &RuleSelection,
        options: &BuiltinRuleOptions,
    ) -> Result<Vec<Box<dyn Rule>>, RulesError> {
        let active_rules = resolve_active_rules(selection)?;

        let mut rules: Vec<Box<dyn Rule>> = Vec::with_capacity(active_rules.len());
        for builtin_rule in active_rules {
            rules.push(builtin_rule.build(options)?);
        }

        Ok(rules)
    }
}

pub fn builtin_rule_ids() -> Result<Vec<RuleId>, RulesError> {
    BuiltinRule::ALL
        .into_iter()
        .map(BuiltinRule::rule_id)
        .collect()
}

pub fn resolve_active_rule_ids(selection: &RuleSelection) -> Result<Vec<RuleId>, RulesError> {
    resolve_active_rules(selection)?
        .into_iter()
        .map(BuiltinRule::rule_id)
        .collect::<Result<Vec<_>, _>>()
}

fn resolve_active_rules(selection: &RuleSelection) -> Result<Vec<BuiltinRule>, RulesError> {
    let unknown = selection
        .select()
        .iter()
        .chain(selection.ignore())
        .filter(|requested| BuiltinRule::from_rule_id(requested).is_none())
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(RulesError::unknown_builtin_rule_ids(&unknown));
    }

    let selected_lookup = selection
        .select()
        .iter()
        .filter_map(BuiltinRule::from_rule_id)
        .collect::<BTreeSet<_>>();
    let ignored_lookup = selection
        .ignore()
        .iter()
        .filter_map(BuiltinRule::from_rule_id)
        .collect::<BTreeSet<_>>();

    let selected_base = if selected_lookup.is_empty() {
        BuiltinRule::ALL.to_vec()
    } else {
        BuiltinRule::ALL
            .into_iter()
            .filter(|rule_id| selected_lookup.contains(rule_id))
            .collect::<Vec<_>>()
    };

    Ok(selected_base
        .into_iter()
        .filter(|rule_id| !ignored_lookup.contains(rule_id))
        .collect())
}

fn normalize_non_empty_trimmed_strings(
    values: impl IntoIterator<Item = String>,
) -> BTreeSet<String> {
    values
        .into_iter()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
        .collect()
}

pub fn parse_builtin_rule_id(value: &'static str) -> Result<RuleId, RulesError> {
    RuleId::parse(value).map_err(|source| RulesError::invalid_builtin_rule_id(value, source))
}

pub fn package_path_from_context(context: &AnalysisContext) -> PathBuf {
    if let Some(target) = context.target() {
        return PathBuf::from(target.package_path());
    }

    context
        .index()
        .source_root()
        .file_name()
        .map_or_else(PathBuf::new, PathBuf::from)
}

pub fn test_root_display_from_context(context: &AnalysisContext) -> PathBuf {
    if let Some(target) = context.target() {
        return PathBuf::from(target.test_root_display());
    }

    context
        .index()
        .test_root()
        .file_name()
        .map_or_else(PathBuf::new, PathBuf::from)
}

pub fn known_target_package_paths_from_context(context: &AnalysisContext) -> Vec<PathBuf> {
    context
        .target()
        .map(|target| {
            target
                .known_target_package_paths()
                .iter()
                .map(PathBuf::from)
                .collect::<Vec<_>>()
        })
        .unwrap_or_default()
}

#[must_use]
pub fn starts_with_path_prefix(test_file: &Path, prefix: &Path) -> bool {
    let prefix_parts = prefix.components().collect::<Vec<_>>();
    let test_parts = test_file.components().collect::<Vec<_>>();
    if test_parts.len() < prefix_parts.len() {
        return false;
    }

    test_parts[..prefix_parts.len()] == prefix_parts
}

#[must_use]
pub fn is_non_unit_test_path(test_file: &Path) -> bool {
    test_file.components().any(|component| {
        component
            .as_os_str()
            .to_str()
            .is_some_and(|segment| segment == "integration" || segment == "e2e")
    })
}

#[must_use]
pub fn is_unit_test_filename(file_name: &str) -> bool {
    file_name.starts_with("test_")
        && Path::new(file_name)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("py"))
}

#[must_use]
pub fn path_to_forward_slashes(path: &Path) -> String {
    path.to_string_lossy().replace('\\', "/")
}
