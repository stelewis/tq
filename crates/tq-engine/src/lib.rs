pub(crate) mod context;
mod error;
pub(crate) mod models;
mod planner;
mod rule_id;
mod runner;

pub use context::{AnalysisContext, PlannedTargetRun, TargetContext, TargetPlanInput};
pub use error::EngineError;
pub use models::{EngineResult, Finding, FindingSummary, Severity};
pub use planner::plan_target_runs;
pub use rule_id::RuleId;
pub use runner::{Rule, RuleEngine, aggregate_results, validate_unique_rule_ids};
