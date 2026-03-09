use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tq_engine::{AnalysisContext, Finding, Rule, RuleId, Severity};

use crate::builtin::{
    is_non_unit_test_path, is_unit_test_filename, package_path_from_context,
    path_to_forward_slashes, starts_with_path_prefix,
};
use crate::error::RulesError;
use crate::qualifiers::{QualifierStrategy, candidate_module_names};

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
            return Err(RulesError::validation(
                "allowed_qualifiers must be non-empty for allowlist strategy",
            ));
        }

        let rule_id = RuleId::parse("orphaned-test")
            .map_err(|error| RulesError::validation(error.to_string()))?;

        Ok(Self {
            rule_id,
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
        let prefix_len = package_path.components().count();
        let test_parts = test_file.components().collect::<Vec<_>>();
        let relative_parts = test_parts
            .iter()
            .skip(prefix_len)
            .take(test_parts.len().saturating_sub(prefix_len + 1))
            .copied()
            .collect::<Vec<_>>();
        let relative_source_dir = relative_parts
            .iter()
            .fold(PathBuf::new(), |path, component| {
                path.join(component.as_os_str())
            });

        let module_stem = test_file
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .and_then(|stem| stem.strip_prefix("test_"))
            .unwrap_or_default();
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
    fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    fn evaluate(&self, context: &AnalysisContext) -> Vec<Finding> {
        let package_path = package_path_from_context(context);
        let source_files = context
            .index()
            .source_files()
            .iter()
            .cloned()
            .collect::<BTreeSet<_>>();
        let mut findings = Vec::new();

        for test_file in context.index().test_files() {
            if is_non_unit_test_path(test_file) {
                continue;
            }

            let Some(file_name) = test_file.file_name().and_then(std::ffi::OsStr::to_str) else {
                continue;
            };
            if !is_unit_test_filename(file_name) {
                continue;
            }

            if !starts_with_path_prefix(test_file, &package_path) {
                continue;
            }

            if self.has_corresponding_source(test_file, &source_files, &package_path) {
                continue;
            }

            if let Ok(finding) = Finding::new(
                self.rule_id.clone(),
                Severity::Warning,
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
            ) {
                findings.push(finding);
            }
        }

        findings
    }
}
