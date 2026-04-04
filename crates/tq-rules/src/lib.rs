pub(crate) mod builtin;
mod error;
mod file_too_large;
mod mapping_missing_test;
mod orphaned_test;
mod qualifiers;
mod structure_mismatch;

pub use builtin::{
    BuiltinRuleOptions, BuiltinRuleRegistry, RuleSelection, builtin_rule_ids,
    resolve_active_rule_ids,
};
pub use error::RulesError;
pub use file_too_large::TestFileTooLargeRule;
pub use mapping_missing_test::MappingMissingTestRule;
pub use orphaned_test::OrphanedTestRule;
pub use qualifiers::candidate_module_names;
pub use structure_mismatch::StructureMismatchRule;
pub use tq_core::QualifierStrategy;
