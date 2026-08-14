//! Cleanup of repository-generated developer artifacts.

use std::path::Path;

use crate::action::{ActionPlan, PlannedAction};

const GENERATED_PATHS: [&str; 3] = ["dist", "docs/.vitepress/dist", "target/cargo-tools"];

#[must_use]
pub fn plan(repo_root: &Path) -> ActionPlan {
    ActionPlan::new(
        "Developer environment cleanup",
        GENERATED_PATHS
            .iter()
            .map(|path| {
                PlannedAction::remove_path(
                    "Remove repository-generated artifact",
                    repo_root.join(path),
                )
            })
            .collect(),
    )
}

#[cfg(test)]
mod tests {
    use std::path::Path;

    use super::plan;
    use crate::action::Action;

    #[test]
    fn cleanup_is_confined_to_owned_repository_outputs() {
        let root = Path::new("/repo");
        let paths = plan(root)
            .actions
            .into_iter()
            .map(|action| match action.action {
                Action::RemovePath { path } => path,
                Action::Command { .. } | Action::ReplaceText { .. } => {
                    panic!("cleanup must only remove explicit paths")
                }
            })
            .collect::<Vec<_>>();

        assert_eq!(
            paths,
            [
                root.join("dist"),
                root.join("docs/.vitepress/dist"),
                root.join("target/cargo-tools"),
            ]
        );
        assert!(paths.iter().all(|path| path.starts_with(root)));
    }
}
