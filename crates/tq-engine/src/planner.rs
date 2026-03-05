use tq_discovery::build_analysis_index;

use crate::context::path_to_forward_slashes;
use crate::{AnalysisContext, EngineError, PlannedTargetRun, TargetContext, TargetPlanInput};

pub fn plan_target_runs(
    configured_targets: &[TargetPlanInput],
    active_targets: &[TargetPlanInput],
) -> Result<Vec<PlannedTargetRun>, EngineError> {
    let known_target_package_paths = configured_targets
        .iter()
        .map(|target| path_to_forward_slashes(target.package_path()))
        .collect::<Vec<_>>();

    let mut planned_runs = Vec::with_capacity(active_targets.len());
    for target in active_targets {
        let index = build_analysis_index(target.source_package_root(), target.test_root())
            .map_err(EngineError::Discovery)?;

        let target_context = TargetContext::new(
            target.name(),
            path_to_forward_slashes(target.package_path()),
            known_target_package_paths.clone(),
            target
                .test_root()
                .file_name()
                .and_then(std::ffi::OsStr::to_str)
                .unwrap_or_default(),
        )?;

        planned_runs.push(PlannedTargetRun::new(
            target.clone(),
            AnalysisContext::with_target(index, target_context),
        ));
    }

    Ok(planned_runs)
}
