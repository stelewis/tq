use std::num::NonZeroU64;

use tq_engine::{AnalysisContext, Finding, Rule, RuleId};

use crate::builtin::BuiltinRule;
use crate::error::RulesError;

pub struct TestFileTooLargeRule {
    rule_id: RuleId,
    max_non_blank_lines: NonZeroU64,
}

impl TestFileTooLargeRule {
    #[must_use]
    pub const fn new(max_non_blank_lines: NonZeroU64) -> Self {
        Self {
            rule_id: BuiltinRule::TestFileTooLarge.rule_id(),
            max_non_blank_lines,
        }
    }
}

impl Rule for TestFileTooLargeRule {
    type Error = RulesError;

    fn rule_id(&self) -> &RuleId {
        &self.rule_id
    }

    fn evaluate(&self, context: &AnalysisContext) -> Result<Vec<Finding>, Self::Error> {
        let mut findings = Vec::new();

        for test_file in context.index().test_files() {
            let line_count = test_file.non_blank_non_comment_lines();
            if line_count <= self.max_non_blank_lines.get() {
                continue;
            }

            findings.push(Finding::new(
                self.rule_id.clone(),
                BuiltinRule::TestFileTooLarge.default_severity(),
                format!(
                    "Test file is too large ({line_count} lines, limit: {})",
                    self.max_non_blank_lines
                ),
                context.index().test_root().join(test_file.path()),
                None,
                Some("Split this module into smaller focused test files".to_owned()),
                None,
            )?);
        }

        Ok(findings)
    }
}
