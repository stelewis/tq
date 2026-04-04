use thiserror::Error;
use tq_core::RuleIdError;
use tq_engine::RuleId;

#[derive(Debug, Error)]
pub enum RulesError {
    #[error("allowed_qualifiers must be non-empty for allowlist strategy")]
    AllowlistRequiresQualifiers,
    #[error("{setting} must be >= 1")]
    ValueMustBePositive { setting: &'static str },
    #[error("invalid built-in rule id definition `{id}`: {source}")]
    InvalidBuiltinRuleId {
        id: &'static str,
        #[source]
        source: RuleIdError,
    },
    #[error("Unknown built-in rule ID(s): {ids}")]
    UnknownBuiltinRuleIds { ids: String },
    #[error("Rules registry defines duplicate built-in rule IDs")]
    DuplicateBuiltinRuleIds,
}

impl RulesError {
    pub(crate) const fn invalid_builtin_rule_id(id: &'static str, source: RuleIdError) -> Self {
        Self::InvalidBuiltinRuleId { id, source }
    }

    pub(crate) const fn allowlist_requires_qualifiers() -> Self {
        Self::AllowlistRequiresQualifiers
    }

    pub(crate) const fn value_must_be_positive(setting: &'static str) -> Self {
        Self::ValueMustBePositive { setting }
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
