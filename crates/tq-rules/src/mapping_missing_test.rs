use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tq_engine::{AnalysisContext, Finding, Rule, RuleId, Severity};

use crate::QualifierStrategy;
use crate::builtin::{package_path_from_context, parse_builtin_rule_id, path_to_forward_slashes};
use crate::candidate_module_names;
use crate::error::RulesError;

pub struct MappingMissingTestRule {
    rule_id: RuleId,
    ignore_init_modules: bool,
    qualifier_strategy: QualifierStrategy,
    allowed_qualifiers: BTreeSet<String>,
}

impl MappingMissingTestRule {
    pub fn new(
        ignore_init_modules: bool,
        qualifier_strategy: QualifierStrategy,
        allowed_qualifiers: BTreeSet<String>,
    ) -> Result<Self, RulesError> {
        if qualifier_strategy == QualifierStrategy::Allowlist && allowed_qualifiers.is_empty() {
            return Err(RulesError::validation(
                "allowed_qualifiers must be non-empty for allowlist strategy",
            ));
        }

        Ok(Self {
            rule_id: parse_builtin_rule_id("mapping-missing-test")?,
            ignore_init_modules,
            qualifier_strategy,
            allowed_qualifiers,
        })
    }

    fn has_matching_test(
        &self,
        source_file: &Path,
        test_files: &[PathBuf],
        package_path: &Path,
    ) -> bool {
        let expected_path = expected_test_path(source_file, package_path);
        let source_stem = source_file
            .file_stem()
            .and_then(std::ffi::OsStr::to_str)
            .unwrap_or_default();

        for test_file in test_files {
            if test_file.parent() != expected_path.parent() {
                continue;
            }

            let Some(file_name) = test_file.file_name().and_then(std::ffi::OsStr::to_str) else {
                continue;
            };
            if !file_name.starts_with("test_") {
                continue;
            }

            let module_stem = test_file
                .file_stem()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or_default()
                .strip_prefix("test_")
                .unwrap_or_default();
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
    fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    fn evaluate(&self, context: &AnalysisContext) -> Vec<Finding> {
        let package_path = package_path_from_context(context);
        let mut findings = Vec::new();

        for source_file in context.index().source_files() {
            if self.ignore_init_modules
                && source_file
                    .file_name()
                    .is_some_and(|name| name == std::ffi::OsStr::new("__init__.py"))
            {
                continue;
            }

            if self.has_matching_test(source_file, context.index().test_files(), &package_path) {
                continue;
            }

            let expected_test_path = expected_test_path(source_file, &package_path);
            if let Ok(finding) = Finding::new(
                self.rule_id.clone(),
                Severity::Error,
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
            ) {
                findings.push(finding);
            }
        }

        findings
    }
}

fn expected_test_path(source_file: &Path, package_path: &Path) -> PathBuf {
    let stem = if source_file
        .file_name()
        .is_some_and(|name| name == std::ffi::OsStr::new("__init__.py"))
    {
        "test___init__".to_owned()
    } else {
        format!(
            "test_{}",
            source_file
                .file_stem()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or_default()
        )
    };

    package_path
        .join(source_file.parent().unwrap_or_else(|| Path::new("")))
        .join(format!("{stem}.py"))
}
