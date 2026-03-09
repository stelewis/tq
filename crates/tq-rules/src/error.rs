use thiserror::Error;
use tq_engine::RuleId;

#[derive(Debug, Error)]
pub enum RulesError {
    #[error("{message}")]
    Validation { message: String },
    #[error("Unknown built-in rule ID(s): {ids}")]
    UnknownBuiltinRuleIds { ids: String },
    #[error("Rules registry defines duplicate built-in rule IDs")]
    DuplicateBuiltinRuleIds,
}

impl RulesError {
    pub(crate) fn validation(message: impl Into<String>) -> Self {
        Self::Validation {
            message: message.into(),
        }
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
