use std::collections::BTreeSet;

use crate::context::path_to_forward_slashes;
use crate::{AnalysisContext, EngineResult, Finding, RuleId, Severity};

pub trait Rule {
    fn rule_id(&self) -> &RuleId;
    fn evaluate(&self, context: &AnalysisContext) -> Vec<Finding>;
}

#[derive(Default)]
pub struct RuleEngine {
    rules: Vec<Box<dyn Rule>>,
}

impl RuleEngine {
    #[must_use]
    pub fn new(rules: Vec<Box<dyn Rule>>) -> Self {
        Self { rules }
    }

    #[must_use]
    pub fn run(&self, context: &AnalysisContext) -> EngineResult {
        let mut findings = Vec::new();

        for rule in &self.rules {
            let _ = rule.rule_id();
            let rule_findings = rule.evaluate(context);
            findings.extend(
                rule_findings
                    .iter()
                    .map(|finding| finding.with_target_if_missing(context.target())),
            );
        }

        findings.sort_by_key(finding_sort_key);
        EngineResult::new(findings)
    }
}

#[must_use]
pub fn aggregate_results(results: &[EngineResult]) -> EngineResult {
    let mut findings = Vec::new();
    for result in results {
        findings.extend(result.findings().iter().cloned());
    }
    findings.sort_by_key(finding_sort_key);
    EngineResult::new(findings)
}

fn finding_sort_key(finding: &Finding) -> (String, String, u32, Severity, String, usize, String) {
    (
        finding.target().unwrap_or_default().to_owned(),
        path_to_forward_slashes(finding.path()),
        finding.line().unwrap_or(0),
        finding.severity(),
        finding.rule_id().as_str().to_owned(),
        finding.message().len(),
        finding.message().to_owned(),
    )
}

#[must_use]
pub fn validate_unique_rule_ids(rule_ids: &[RuleId]) -> bool {
    let mut seen = BTreeSet::new();
    for rule_id in rule_ids {
        if !seen.insert(rule_id) {
            return false;
        }
    }
    true
}
