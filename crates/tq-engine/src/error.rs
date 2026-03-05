use thiserror::Error;
use tq_discovery::DiscoveryError;

#[derive(Debug, Error)]
pub enum EngineError {
    #[error("{message}")]
    Validation { message: String },
    #[error(transparent)]
    Discovery(#[from] DiscoveryError),
}
