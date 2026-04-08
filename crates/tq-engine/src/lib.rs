pub(crate) mod context;
mod error;
pub(crate) mod models;
mod planner;
mod runner;

pub use context::{AnalysisContext, PlannedTargetRun, TargetContext, TargetPlanInput};
pub use error::EngineError;
pub use models::{EngineResult, Finding, FindingSummary};
pub use planner::plan_target_runs;
pub use runner::{Rule, RuleEngine, aggregate_results, validate_unique_rule_ids};
pub use tq_core::{RuleId, Severity};
