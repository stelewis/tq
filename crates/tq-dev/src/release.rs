//! Local release artifact builds.

use std::path::Path;

use crate::action::{ActionPlan, PlannedAction};
use crate::error::DevError;
use crate::invocation::Invocation;
use crate::manifest::DevToolsManifest;

/// Exact Python build tool requirements used by release artifact builders.
#[derive(Debug, Eq, PartialEq)]
pub struct ReleaseBuildToolRequirements {
    pub maturin: String,
    pub maturin_zig: String,
}

impl ReleaseBuildToolRequirements {
    #[must_use]
    pub fn github_output(&self) -> String {
        format!(
            "maturin={}\nmaturin_zig={}\n",
            self.maturin, self.maturin_zig
        )
    }
}

/// Resolves the exact Python build tool requirements from the dev-tools manifest.
pub fn build_tool_requirements(repo_root: &Path) -> Result<ReleaseBuildToolRequirements, DevError> {
    let manifest = DevToolsManifest::load(repo_root)?;
    Ok(ReleaseBuildToolRequirements {
        maturin: format!("maturin=={}", manifest.maturin),
        maturin_zig: format!("maturin[zig]=={}", manifest.maturin),
    })
}

/// The release artifact build plan: clear `dist/`, then build the sdist and
/// host wheel.
pub fn plan(repo_root: &Path) -> Result<ActionPlan, DevError> {
    let tools = build_tool_requirements(repo_root)?;
    Ok(ActionPlan::new(
        "Release artifact build",
        vec![
            PlannedAction::remove_path("Remove previous release artifacts", repo_root.join("dist")),
            PlannedAction::command(
                "Build Python source distribution",
                Invocation::new("uv", ["build", "--sdist"]),
            ),
            PlannedAction::command(
                "Build host release wheel",
                Invocation::with_args(
                    "uv",
                    vec![
                        "run".to_owned(),
                        "--isolated".to_owned(),
                        "--with".to_owned(),
                        tools.maturin,
                        "--".to_owned(),
                        "maturin".to_owned(),
                        "build".to_owned(),
                        "--release".to_owned(),
                        "--locked".to_owned(),
                        "--manifest-path".to_owned(),
                        "crates/tq-cli/Cargo.toml".to_owned(),
                        "--bindings".to_owned(),
                        "bin".to_owned(),
                        "--out".to_owned(),
                        "dist".to_owned(),
                        "-i".to_owned(),
                        "python".to_owned(),
                    ],
                ),
            ),
        ],
    ))
}
