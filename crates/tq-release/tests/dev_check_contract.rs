use std::fs;
use std::path::Path;

use tq_release::{DevAction, DevCheckProfile, DevCheckTarget};

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("parent path must exist"))
        .expect("create parent directories");
    fs::write(path, contents).expect("write file");
}

fn write_dev_tools_manifest(repo_root: &Path) {
    write(
        &repo_root.join(".github/dev-tools.toml"),
        concat!(
            "[schema]\n",
            "version = 1\n",
            "\n",
            "[tools]\n",
            "rust = \"1.96.1\"\n",
            "python = \"3.14.6\"\n",
            "uv = \"0.11.28\"\n",
            "node = \"26.4.0\"\n",
            "npm = \"11.17.0\"\n",
            "mise = \"2026.7.5\"\n",
            "\n",
            "[rust-maintenance]\n",
            "cargo-outdated = \"0.17.0\"\n",
            "cargo-audit = \"0.22.1\"\n",
            "cargo-deny = \"0.19.0\"\n",
        ),
    );
}

#[test]
fn routine_check_plan_is_the_fast_daily_gate() {
    let plan = tq_release::plan_dev_checks(DevCheckTarget::Routine, DevCheckProfile::Fast);
    let task_ids = plan
        .tasks
        .iter()
        .map(|task| task.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(task_ids, ["rust-format", "rust-lint", "rust-tests"]);
}

#[test]
fn all_check_plan_keeps_release_packaging_in_the_full_profile() {
    let fast_plan = tq_release::plan_dev_checks(DevCheckTarget::All, DevCheckProfile::Fast);
    let full_plan = tq_release::plan_dev_checks(DevCheckTarget::All, DevCheckProfile::Full);

    let fast_task_ids = fast_plan
        .tasks
        .iter()
        .map(|task| task.id.as_str())
        .collect::<Vec<_>>();
    let full_task_ids = full_plan
        .tasks
        .iter()
        .map(|task| task.id.as_str())
        .collect::<Vec<_>>();

    assert_eq!(
        fast_task_ids,
        [
            "rust-format",
            "rust-lint",
            "rust-tests",
            "docs-sync",
            "release-policy"
        ]
    );
    assert_eq!(
        full_task_ids,
        [
            "rust-format",
            "rust-lint",
            "rust-tests",
            "docs-sync",
            "release-policy",
            "cargo-package",
            "release-build"
        ]
    );
}

#[test]
fn release_build_plan_exposes_the_artifact_build_commands() {
    let plan = tq_release::plan_release_artifacts();
    let commands = plan
        .commands
        .iter()
        .map(|command| command.command.display())
        .collect::<Vec<_>>();

    assert_eq!(
        commands,
        [
            "uv build --sdist",
            "uv run --isolated --with maturin>=1.11,<2.0 -- maturin build --release --locked --manifest-path crates/tq-cli/Cargo.toml --bindings bin --out dist -i python"
        ]
    );
}

#[test]
fn dependency_update_plan_exposes_file_and_command_actions() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_dev_tools_manifest(temp.path());

    let plan = tq_release::plan_update_dev_dependencies(temp.path())
        .expect("dependency update plan should build");
    let commands = plan
        .actions
        .iter()
        .filter_map(|action| match &action.action {
            DevAction::Command { command } => Some(command.display()),
            DevAction::ReplaceText { .. } | DevAction::RemovePath { .. } => None,
        })
        .collect::<Vec<_>>();

    assert!(
        commands
            .iter()
            .any(|command| command.starts_with("rustup update "))
    );
    assert!(commands.contains(&"uv lock --upgrade".to_owned()));
    assert!(commands.contains(&"npm update".to_owned()));
}

#[test]
fn cleanup_plan_exposes_cache_removal_action() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_dev_tools_manifest(temp.path());

    let plan = tq_release::plan_cleanup_dev_environment(temp.path());

    assert_eq!(plan.actions.len(), 1);
    assert!(matches!(
        &plan.actions[0].action,
        DevAction::RemovePath { path } if path == &temp.path().join("target/cargo-tools")
    ));
}
