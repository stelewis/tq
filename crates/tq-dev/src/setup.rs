//! Repository-local developer setup.

use crate::action::{ActionPlan, PlannedAction};
use crate::invocation::Invocation;

/// Installs locked project dependencies and repository hooks.
#[must_use]
pub fn plan() -> ActionPlan {
    ActionPlan::new(
        "Developer setup",
        vec![
            PlannedAction::command(
                "Sync locked Python dependencies",
                Invocation::new("uv", ["sync", "--locked"]),
            ),
            PlannedAction::command(
                "Install locked npm dependencies",
                Invocation::new("npm", ["ci", "--ignore-scripts"]),
            ),
            PlannedAction::command(
                "Install pre-commit hooks",
                Invocation::new("uv", ["run", "prek", "install", "--install-hooks"]),
            ),
        ],
    )
}
