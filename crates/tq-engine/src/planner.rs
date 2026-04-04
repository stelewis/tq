use tq_discovery::build_analysis_index;

use crate::{AnalysisContext, EngineError, PlannedTargetRun, TargetContext, TargetPlanInput};

pub fn plan_target_runs(
    configured_targets: &[TargetPlanInput],
    active_targets: &[TargetPlanInput],
) -> Result<Vec<PlannedTargetRun>, EngineError> {
    let known_target_package_paths = configured_targets
        .iter()
        .map(|target| target.package_path().clone())
        .collect::<Vec<_>>();

    let mut planned_runs = Vec::with_capacity(active_targets.len());
    for target in active_targets {
        let index = build_analysis_index(target.source_package_root(), target.test_root())
            .map_err(EngineError::Discovery)?;

        let target_context = TargetContext::new(
            target.name().clone(),
            target.package_path().clone(),
            known_target_package_paths.clone(),
            target.test_root_display().to_path_buf(),
        );

        planned_runs.push(PlannedTargetRun::new(
            target.clone(),
            AnalysisContext::with_target(index, target_context),
        ));
    }

    Ok(planned_runs)
}
