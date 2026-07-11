//! Local release artifact builds.

use std::path::Path;

use crate::action::{ActionPlan, PlannedAction};
use crate::invocation::Invocation;

/// The release artifact build plan: clear `dist/`, then build the sdist and
/// host wheel.
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
