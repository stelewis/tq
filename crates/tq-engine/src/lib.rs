pub(crate) mod context;
mod error;
pub(crate) mod models;
mod runner;

pub use context::{AnalysisContext, TargetContext};
pub use error::EngineError;
pub use models::{EngineResult, Finding, FindingSummary};
pub use runner::{Rule, RuleEngine, aggregate_results, validate_unique_rule_ids};
pub use tq_core::{RuleId, Severity};
