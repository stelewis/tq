use std::path::Path;

use tq_core::accepted::AcceptedFinding;
use tq_engine::Finding;

use crate::builtin::BuiltinRule;
use crate::error::RulesError;

#[derive(Clone, Copy)]
pub enum UnusedAcceptedFindings {
    Report,
    Ignore,
}

pub fn unused_findings<'a>(
    entries: impl Iterator<Item = &'a AcceptedFinding>,
    root: &Path,
    reporting: UnusedAcceptedFindings,
) -> Result<Vec<Finding>, RulesError> {
    if matches!(reporting, UnusedAcceptedFindings::Ignore) {
        return Ok(Vec::new());
    }
    entries
        .map(|entry| {
            let rationale = entry
                .rationale()
                .map_or_else(String::new, |text| format!(" (rationale: {text})"));
            Finding::new(
                BuiltinRule::UnusedAcceptedFinding.rule_id(),
                BuiltinRule::UnusedAcceptedFinding.default_severity(),
                format!(
                    "Accepted finding is unused: {} no longer applies{rationale}",
                    entry.kind().rule_id(),
                ),
                root.join(entry.path()),
                None,
                Some("Remove this entry from accepted_findings".to_owned()),
                None,
            )
            .map_err(RulesError::from)
        })
        .collect()
}
