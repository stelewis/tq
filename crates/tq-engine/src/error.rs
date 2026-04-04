use thiserror::Error;
use tq_discovery::DiscoveryError;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("Rule engine received duplicate rule ids")]
    DuplicateRuleIds,
    #[error("Finding message must be non-empty")]
    EmptyFindingMessage,
    #[error("Finding line must be >= 1 when provided")]
    InvalidFindingLine,
    #[error(transparent)]
    Discovery(#[from] DiscoveryError),
}
