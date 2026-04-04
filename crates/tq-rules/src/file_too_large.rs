use std::fs::File;
use std::io::{self, BufRead, BufReader};

use tq_engine::{AnalysisContext, Finding, Rule, RuleId, Severity};

use crate::builtin::{parse_builtin_rule_id, path_to_forward_slashes};
use crate::error::RulesError;

pub struct TestFileTooLargeRule {
    rule_id: RuleId,
    max_non_blank_lines: u64,
}

impl TestFileTooLargeRule {
    pub fn new(max_non_blank_lines: u64) -> Result<Self, RulesError> {
        if max_non_blank_lines < 1 {
            return Err(RulesError::value_must_be_positive("max_non_blank_lines"));
        }

        Ok(Self {
            rule_id: parse_builtin_rule_id("test-file-too-large")?,
            max_non_blank_lines,
        })
    }
}

impl Rule for TestFileTooLargeRule {
    fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    fn evaluate(&self, context: &AnalysisContext) -> Vec<Finding> {
        let mut findings = Vec::new();

        for test_file in context.index().test_files() {
            let full_path = context.index().test_root().join(test_file);
            match count_non_blank_non_comment_lines(&full_path) {
                Ok(line_count) => {
                    if line_count <= self.max_non_blank_lines {
                        continue;
                    }

                    if let Ok(finding) = Finding::new(
                        self.rule_id.clone(),
                        Severity::Warning,
                        format!(
                            "Test file is too large ({line_count} lines, limit: {})",
                            self.max_non_blank_lines
                        ),
                        full_path,
                        None,
                        Some("Split this module into smaller focused test files".to_owned()),
                        None,
                    ) {
                        findings.push(finding);
                    }
                }
                Err(_) => {
                    if let Ok(finding) = Finding::new(
                        self.rule_id.clone(),
                        Severity::Warning,
                        format!(
                            "Could not read test file for size check (path: {})",
                            path_to_forward_slashes(test_file)
                        ),
                        full_path,
                        None,
                        Some("Ensure file exists and is UTF-8 decodable".to_owned()),
                        None,
                    ) {
                        findings.push(finding);
                    }
                }
            }
        }

        findings
    }
}

fn count_non_blank_non_comment_lines(path: &std::path::Path) -> io::Result<u64> {
    let file = File::open(path)?;
    let reader = BufReader::new(file);

    let mut line_count = 0;
    for line in reader.lines() {
        let line = line?;
        let stripped = line.trim();
        if stripped.is_empty() || stripped.starts_with('#') {
            continue;
        }
        line_count += 1;
    }

    Ok(line_count)
}
