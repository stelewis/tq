use std::collections::{BTreeMap, BTreeSet};
use std::path::{Path, PathBuf};

use tq_engine::{AnalysisContext, Rule, RuleId};

use crate::error::RulesError;
use crate::file_too_large::TestFileTooLargeRule;
use crate::mapping_missing_test::MappingMissingTestRule;
use crate::orphaned_test::OrphanedTestRule;
use crate::qualifiers::QualifierStrategy;
use crate::structure_mismatch::StructureMismatchRule;

const MAPPING_MISSING_TEST: &str = "mapping-missing-test";
const STRUCTURE_MISMATCH: &str = "structure-mismatch";
const TEST_FILE_TOO_LARGE: &str = "test-file-too-large";
const ORPHANED_TEST: &str = "orphaned-test";

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
        let selected_rule_ids = resolve_active_rule_ids(selection)?;

        let mut rules: Vec<Box<dyn Rule>> = Vec::with_capacity(selected_rule_ids.len());
        for rule_id in selected_rule_ids {
            match rule_id.as_str() {
                MAPPING_MISSING_TEST => {
                    rules.push(Box::new(MappingMissingTestRule::new(
                        options.ignore_init_modules(),
                        options.qualifier_strategy(),
                        options.allowed_qualifiers().clone(),
                    )?));
                }
                STRUCTURE_MISMATCH => {
                    rules.push(Box::new(StructureMismatchRule::new()?));
                }
                TEST_FILE_TOO_LARGE => {
                    rules.push(Box::new(TestFileTooLargeRule::new(
                        options.max_test_file_non_blank_lines(),
                    )?));
                }
                ORPHANED_TEST => {
                    rules.push(Box::new(OrphanedTestRule::new(
                        options.qualifier_strategy(),
                        options.allowed_qualifiers().clone(),
                    )?));
                }
                _ => {
                    return Err(RulesError::validation(format!(
                        "unsupported built-in rule id: {}",
                        rule_id.as_str()
                    )));
                }
            }
        }

        Ok(rules)
    }
}

pub fn builtin_rule_ids() -> Result<Vec<RuleId>, RulesError> {
    let ids = [
        RuleId::parse(MAPPING_MISSING_TEST),
        RuleId::parse(STRUCTURE_MISMATCH),
        RuleId::parse(TEST_FILE_TOO_LARGE),
        RuleId::parse(ORPHANED_TEST),
    ];

    let mut parsed = Vec::with_capacity(ids.len());
    for result in ids {
        let rule_id = result.map_err(|error| RulesError::validation(error.to_string()))?;
        parsed.push(rule_id);
    }

    validate_unique_builtin_rule_ids(&parsed)?;
    Ok(parsed)
}

pub fn resolve_active_rule_ids(selection: &RuleSelection) -> Result<Vec<RuleId>, RulesError> {
    let builtins = builtin_rule_ids()?;

    let by_id = builtins
        .iter()
        .cloned()
        .map(|rule_id| (rule_id.as_str().to_owned(), rule_id))
        .collect::<BTreeMap<_, _>>();

    let unknown = selection
        .select()
        .iter()
        .chain(selection.ignore())
        .filter(|requested| !by_id.contains_key(requested.as_str()))
        .cloned()
        .collect::<Vec<_>>();
    if !unknown.is_empty() {
        return Err(RulesError::unknown_builtin_rule_ids(&unknown));
    }

    let selected_lookup = selection
        .select()
        .iter()
        .map(RuleId::as_str)
        .collect::<BTreeSet<_>>();
    let ignored_lookup = selection
        .ignore()
        .iter()
        .map(RuleId::as_str)
        .collect::<BTreeSet<_>>();

    let selected_base = if selected_lookup.is_empty() {
        builtins
    } else {
        builtins
            .into_iter()
            .filter(|rule_id| selected_lookup.contains(rule_id.as_str()))
            .collect::<Vec<_>>()
    };

    Ok(selected_base
        .into_iter()
        .filter(|rule_id| !ignored_lookup.contains(rule_id.as_str()))
        .collect())
}

fn validate_unique_builtin_rule_ids(rule_ids: &[RuleId]) -> Result<(), RulesError> {
    let mut seen = BTreeSet::new();
    for rule_id in rule_ids {
        if !seen.insert(rule_id.as_str()) {
            return Err(RulesError::DuplicateBuiltinRuleIds);
        }
    }
    Ok(())
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
