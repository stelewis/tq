//! Local release artifact builds.

use std::path::Path;

use crate::action::{ActionPlan, PlannedAction};
use crate::error::DevError;
use crate::invocation::Invocation;

/// The plan is the single source of truth: `run` clears `dist/` and applies
/// exactly this plan.
#[must_use]
pub fn plan(repo_root: &Path) -> ActionPlan {
    ActionPlan::new(
        "Release artifact build",
        vec![
            PlannedAction::remove_path("Remove previous release artifacts", repo_root.join("dist")),
            PlannedAction::command(
                "Build Python source distribution",
                Invocation::new("uv", ["build", "--sdist"]),
            ),
            PlannedAction::command(
                "Build host release wheel",
                Invocation::new(
                    "uv",
                    [
                        "run",
                        "--isolated",
                        "--with",
                        "maturin>=1.11,<2.0",
                        "--",
                        "maturin",
                        "build",
                        "--release",
                        "--locked",
                        "--manifest-path",
                        "crates/tq-cli/Cargo.toml",
                        "--bindings",
                        "bin",
                        "--out",
                        "dist",
                        "-i",
                        "python",
                    ],
                ),
            ),
        ],
    )
}

pub fn run(repo_root: &Path) -> Result<(), DevError> {
    plan(repo_root).apply(repo_root)
}
