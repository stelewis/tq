use std::path::PathBuf;

use tq_core::RelativePathBuf;
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

        let test_root_name = target
            .test_root()
            .file_name()
            .map(PathBuf::from)
            .ok_or_else(|| EngineError::MissingTestRootDisplay {
                path: target.test_root().to_path_buf(),
            })?;
        let test_root_display = RelativePathBuf::new(test_root_name).map_err(|source| {
            EngineError::InvalidTestRootDisplay {
                path: target.test_root().to_path_buf(),
                source,
            }
        })?;

        let target_context = TargetContext::new(
            target.name().clone(),
            target.package_path().clone(),
            known_target_package_paths.clone(),
            test_root_display,
        );

        planned_runs.push(PlannedTargetRun::new(
            target.clone(),
            AnalysisContext::with_target(index, target_context),
        ));
    }

    Ok(planned_runs)
}
