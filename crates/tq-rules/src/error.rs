use thiserror::Error;
use tq_engine::{EngineError, RuleId};

#[derive(Debug, Error)]
pub enum RulesError {
    #[error("allowed_qualifiers must be non-empty for allowlist strategy")]
    AllowlistRequiresQualifiers,
    #[error("Unknown built-in rule ID(s): {ids}")]
    UnknownBuiltinRuleIds { ids: String },
    #[error("Rule produced an invalid finding: {0}")]
    InvalidFinding(#[from] EngineError),
}

impl RulesError {
    pub(crate) const fn allowlist_requires_qualifiers() -> Self {
        Self::AllowlistRequiresQualifiers
    }

    pub(crate) fn unknown_builtin_rule_ids(rule_ids: &[RuleId]) -> Self {
        let ids = rule_ids
            .iter()
            .map(RuleId::as_str)
            .collect::<Vec<_>>()
            .join(", ");
        Self::UnknownBuiltinRuleIds { ids }
    }
}
