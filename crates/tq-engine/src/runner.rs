use std::collections::BTreeSet;

use tq_core::path_to_forward_slashes;

use crate::{AnalysisContext, EngineError, EngineResult, Finding, RuleId, Severity};

pub trait Rule {
    type Error;

    fn rule_id(&self) -> &RuleId;
    fn evaluate(&self, context: &AnalysisContext) -> Result<Vec<Finding>, Self::Error>;
}

#[derive(Default)]
pub struct RuleEngine<E> {
    rules: Vec<Box<dyn Rule<Error = E>>>,
}

impl<E> RuleEngine<E> {
    pub fn new(rules: Vec<Box<dyn Rule<Error = E>>>) -> Result<Self, EngineError> {
        let rule_ids = rules
            .iter()
            .map(|rule| rule.rule_id().clone())
            .collect::<Vec<_>>();
        if !validate_unique_rule_ids(&rule_ids) {
            return Err(EngineError::DuplicateRuleIds);
        }

        Ok(Self { rules })
    }

    pub fn run(&self, context: &AnalysisContext) -> Result<EngineResult, E> {
        let mut findings = Vec::new();

        for rule in &self.rules {
            let rule_findings = rule.evaluate(context)?;
            findings.extend(
                rule_findings
                    .iter()
                    .map(|finding| finding.with_target_if_missing(context.target())),
            );
        }

        findings.sort_by_key(finding_sort_key);
        Ok(EngineResult::new(findings))
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
        finding
            .target()
            .map_or_else(String::new, ToString::to_string),
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
