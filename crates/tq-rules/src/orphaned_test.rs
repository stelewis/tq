use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tq_core::{path_to_forward_slashes, python_test_module_name, source_directory_for_unit_test};
use tq_engine::{AnalysisContext, Finding, Rule, RuleId};

use crate::QualifierStrategy;
use crate::builtin::{BuiltinRule, is_non_unit_test_path, starts_with_path_prefix};
use crate::candidate_module_names;
use crate::error::RulesError;

pub struct OrphanedTestRule {
    rule_id: RuleId,
    qualifier_strategy: QualifierStrategy,
    allowed_qualifiers: BTreeSet<String>,
}

impl OrphanedTestRule {
    pub fn new(
        qualifier_strategy: QualifierStrategy,
        allowed_qualifiers: BTreeSet<String>,
    ) -> Result<Self, RulesError> {
        if qualifier_strategy == QualifierStrategy::Allowlist && allowed_qualifiers.is_empty() {
            return Err(RulesError::allowlist_requires_qualifiers());
        }

        Ok(Self {
            rule_id: BuiltinRule::OrphanedTest.rule_id(),
            qualifier_strategy,
            allowed_qualifiers,
        })
    }

    fn has_corresponding_source(
        &self,
        test_file: &Path,
        source_files: &BTreeSet<PathBuf>,
        package_path: &Path,
    ) -> bool {
        let Some(relative_source_dir) = source_directory_for_unit_test(test_file, package_path)
        else {
            return false;
        };
        let Some(module_stem) = python_test_module_name(test_file) else {
            return false;
        };

        for module_name in candidate_module_names(
            module_stem,
            self.qualifier_strategy,
            &self.allowed_qualifiers,
        ) {
            let source_file = relative_source_dir.join(format!("{module_name}.py"));
            if source_files.contains(&source_file) {
                return true;
            }
        }

        false
    }
}

impl Rule for OrphanedTestRule {
    type Error = RulesError;

    fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    fn evaluate(&self, context: &AnalysisContext) -> Result<Vec<Finding>, Self::Error> {
        let package_path = context.package_path();
        let source_files = context
            .index()
            .source_files()
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut findings = Vec::new();

        for test_file in context.index().test_files() {
            let test_file = test_file.path();
            if is_non_unit_test_path(test_file) || !tq_core::is_python_test_file(test_file) {
                continue;
            }

            if !starts_with_path_prefix(test_file, package_path) {
                continue;
            }

            if self.has_corresponding_source(test_file, &source_files, package_path) {
                continue;
            }

            findings.push(Finding::new(
                self.rule_id.clone(),
                BuiltinRule::OrphanedTest.default_severity(),
                format!(
                    "Test file has no corresponding source module: {}",
                    path_to_forward_slashes(test_file)
                ),
                context.index().test_root().join(test_file),
                None,
                Some(
                    "Verify this test is still needed or move it to integration/e2e scope"
                        .to_owned(),
                ),
                None,
            )?);
        }

        Ok(findings)
    }
}
