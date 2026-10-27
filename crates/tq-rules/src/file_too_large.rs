use std::collections::BTreeMap;
use std::num::NonZeroU64;

use tq_core::accepted::{AcceptedFinding, AcceptedFindingKind};
use tq_engine::{AnalysisContext, Finding, Rule, RuleId};

use crate::accepted::{UnusedAcceptedFindings, unused_findings};
use crate::builtin::BuiltinRule;
use crate::error::RulesError;

pub struct TestFileTooLargeRule {
    rule_id: RuleId,
    max_non_blank_lines: NonZeroU64,
    accepted: Vec<AcceptedFinding>,
    unused: UnusedAcceptedFindings,
}

impl TestFileTooLargeRule {
    #[must_use]
    pub fn new(
        max_non_blank_lines: NonZeroU64,
        accepted: &[AcceptedFinding],
        unused: UnusedAcceptedFindings,
    ) -> Self {
        Self {
            rule_id: BuiltinRule::TestFileTooLarge.rule_id(),
            max_non_blank_lines,
            accepted: accepted
                .iter()
                .filter(|entry| {
                    matches!(entry.kind(), AcceptedFindingKind::TestFileTooLarge { .. })
                })
                .cloned()
                .collect(),
            unused,
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
        let mut pending: BTreeMap<_, _> = self
            .accepted
            .iter()
            .map(|entry| (entry.path(), entry))
            .collect();

        for test_file in context.index().test_files() {
            let line_count = test_file.non_blank_non_comment_lines();
            if line_count <= self.max_non_blank_lines.get() {
                continue;
            }

            let message = if let Some(entry) = pending.remove(test_file.path())
                && let AcceptedFindingKind::TestFileTooLarge {
                    max_non_blank_lines,
                } = entry.kind()
            {
                if line_count <= max_non_blank_lines.get() {
                    continue;
                }
                format!(
                    "Test file is too large ({line_count} lines, accepted ceiling: {max_non_blank_lines}, limit: {})",
                    self.max_non_blank_lines
                )
            } else {
                format!(
                    "Test file is too large ({line_count} lines, limit: {})",
                    self.max_non_blank_lines
                )
            };
            findings.push(Finding::new(
                self.rule_id.clone(),
                BuiltinRule::TestFileTooLarge.default_severity(),
                message,
                context.index().test_root().join(test_file.path()),
                None,
                Some("Split this module into smaller focused test files".to_owned()),
                None,
            )?);
        }

        findings.extend(unused_findings(
            pending.into_values(),
            context.index().test_root(),
            self.unused,
        )?);
        Ok(findings)
    }
}
