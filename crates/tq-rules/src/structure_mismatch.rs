use std::collections::BTreeSet;
use std::path::{Path, PathBuf};

use tq_core::RelativePathBuf;
use tq_engine::{AnalysisContext, Finding, Rule, RuleId};

use crate::builtin::{
    BuiltinRule, is_non_unit_test_path, is_unit_test_filename, path_to_forward_slashes,
    starts_with_path_prefix,
};

pub struct StructureMismatchRule {
    rule_id: RuleId,
}

impl StructureMismatchRule {
    pub fn new() -> Result<Self, crate::error::RulesError> {
        Ok(Self {
            rule_id: BuiltinRule::StructureMismatch.rule_id()?,
        })
    }
}

impl Rule for StructureMismatchRule {
    fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    fn evaluate(&self, context: &AnalysisContext) -> Vec<Finding> {
        let package_path = context.package_path();
        let test_root_display = context.test_root_display();
        let known_target_paths = context.known_target_package_paths();
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

            if !starts_with_path_prefix(test_file, package_path) {
                if belongs_to_other_target(test_file, package_path, known_target_paths) {
                    continue;
                }

                let suggestion_path = test_root_display.join(package_path).join(file_name);
                if let Ok(finding) = Finding::new(
                    self.rule_id.clone(),
                    BuiltinRule::StructureMismatch.default_severity(),
                    "Unit test is not located under the package test root",
                    context.index().test_root().join(test_file),
                    None,
                    Some(format!(
                        "Move test under: {}",
                        path_to_forward_slashes(&suggestion_path)
                    )),
                    None,
                ) {
                    findings.push(finding);
                }
                continue;
            }

            let Some(expected_path) =
                expected_path_for_test_file(test_file, &source_files, package_path)
            else {
                continue;
            };
            if expected_path == *test_file {
                continue;
            }

            if let Ok(finding) = Finding::new(
                self.rule_id.clone(),
                BuiltinRule::StructureMismatch.default_severity(),
                "Test file is not in the expected location",
                context.index().test_root().join(test_file),
                None,
                Some(format!(
                    "Move to: {}",
                    path_to_forward_slashes(&test_root_display.join(expected_path))
                )),
                None,
            ) {
                findings.push(finding);
            }
        }

        findings
    }
}

fn expected_path_for_test_file(
    test_file: &Path,
    source_files: &BTreeSet<PathBuf>,
    package_path: &Path,
) -> Option<PathBuf> {
    let source_candidate = resolve_source_candidate(test_file, source_files, package_path)?;
    let expected_name = test_file.file_name()?;
    Some(
        package_path
            .join(source_candidate.parent()?)
            .join(expected_name),
    )
}

fn resolve_source_candidate(
    test_file: &Path,
    source_files: &BTreeSet<PathBuf>,
    package_path: &Path,
) -> Option<PathBuf> {
    let module_stem = test_file
        .file_stem()
        .and_then(std::ffi::OsStr::to_str)
        .and_then(|stem| stem.strip_prefix("test_"))?;
    for candidate in candidate_source_paths(test_file, module_stem, package_path) {
        if source_files.contains(&candidate) {
            return Some(candidate);
        }
    }

    let bare_name = format!("{}.py", module_stem.split('_').next().unwrap_or_default());
    let same_name_sources = source_files
        .iter()
        .filter(|path| {
            path.file_name()
                .is_some_and(|name| name == std::ffi::OsStr::new(&bare_name))
        })
        .collect::<Vec<_>>();
    if same_name_sources.len() == 1 {
        return same_name_sources.first().map(|path| (*path).clone());
    }

    None
}

fn candidate_source_paths(
    test_file: &Path,
    module_stem: &str,
    package_path: &Path,
) -> Vec<PathBuf> {
    let prefix_len = package_path.components().count();
    let test_components = test_file.components().collect::<Vec<_>>();
    let relative_components = test_components
        .iter()
        .skip(prefix_len)
        .take(test_components.len().saturating_sub(prefix_len + 1))
        .copied()
        .collect::<Vec<_>>();

    let relative_source_dir = relative_components
        .iter()
        .fold(PathBuf::new(), |path, component| {
            path.join(component.as_os_str())
        });

    let direct_source = relative_source_dir.join(format!("{module_stem}.py"));
    if !module_stem.contains('_') {
        return vec![direct_source];
    }

    let mut candidates = vec![direct_source];
    let parts = module_stem.split('_').collect::<Vec<_>>();
    for index in (1..parts.len()).rev() {
        let prefix = parts[..index].join("_");
        candidates.push(relative_source_dir.join(format!("{prefix}.py")));
    }

    candidates
}

fn belongs_to_other_target(
    test_file: &Path,
    active_package_path: &Path,
    known_target_paths: &[RelativePathBuf],
) -> bool {
    known_target_paths.iter().any(|path| {
        let known_path = path.as_path();
        known_path != active_package_path && starts_with_path_prefix(test_file, known_path)
    })
}
