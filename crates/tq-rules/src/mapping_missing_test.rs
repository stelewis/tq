use std::collections::BTreeSet;
use std::path::Path;

use tq_core::{
    InitModulesMode, path_to_forward_slashes, python_module_name, python_test_module_name,
    unit_test_path_for_source,
};
use tq_discovery::AnalyzedTestFile;
use tq_engine::{AnalysisContext, Finding, Rule, RuleId};

use crate::QualifierStrategy;
use crate::builtin::BuiltinRule;
use crate::candidate_module_names;
use crate::error::RulesError;

pub struct MappingMissingTestRule {
    rule_id: RuleId,
    init_modules: InitModulesMode,
    qualifier_strategy: QualifierStrategy,
    allowed_qualifiers: BTreeSet<String>,
}

impl MappingMissingTestRule {
    pub fn new(
        init_modules: InitModulesMode,
        qualifier_strategy: QualifierStrategy,
        allowed_qualifiers: BTreeSet<String>,
    ) -> Result<Self, RulesError> {
        if qualifier_strategy == QualifierStrategy::Allowlist && allowed_qualifiers.is_empty() {
            return Err(RulesError::allowlist_requires_qualifiers());
        }

        Ok(Self {
            rule_id: BuiltinRule::MappingMissingTest.rule_id(),
            init_modules,
            qualifier_strategy,
            allowed_qualifiers,
        })
    }

    fn has_matching_test(
        &self,
        source_file: &Path,
        test_files: &[AnalyzedTestFile],
        package_path: &Path,
    ) -> bool {
        let Some(expected_path) = unit_test_path_for_source(source_file, package_path) else {
            return false;
        };
        let source_stem = python_module_name(source_file).unwrap_or_default();

        for test_file in test_files {
            let test_file = test_file.path();
            if test_file.parent() != expected_path.parent() {
                continue;
            }

            let Some(module_stem) = python_test_module_name(test_file) else {
                continue;
            };
            let candidates = candidate_module_names(
                module_stem,
                self.qualifier_strategy,
                &self.allowed_qualifiers,
            );
            if candidates.iter().any(|candidate| candidate == source_stem) {
                return true;
            }
        }

        false
    }
}

impl Rule for MappingMissingTestRule {
    type Error = RulesError;

    fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    fn evaluate(&self, context: &AnalysisContext) -> Result<Vec<Finding>, Self::Error> {
        let package_path = context.package_path();
        let mut findings = Vec::new();

        for source_file in context.index().source_files() {
            if matches!(self.init_modules, InitModulesMode::Ignore)
                && source_file
                    .file_name()
                    .is_some_and(|name| name == std::ffi::OsStr::new("__init__.py"))
            {
                continue;
            }

            if self.has_matching_test(source_file, context.index().test_files(), package_path) {
                continue;
            }

            let Some(expected_test_path) = unit_test_path_for_source(source_file, package_path)
            else {
                continue;
            };
            findings.push(Finding::new(
                self.rule_id.clone(),
                BuiltinRule::MappingMissingTest.default_severity(),
                format!(
                    "No test file found for source module: {}",
                    path_to_forward_slashes(source_file)
                ),
                context.index().source_root().join(source_file),
                None,
                Some(format!(
                    "Create test file at: {}",
                    path_to_forward_slashes(&expected_test_path)
                )),
                None,
            )?);
        }

        Ok(findings)
    }
}
