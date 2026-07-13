use std::fs;
use std::path::Path;

use tq_dev::action::Action;
use tq_dev::check::{CheckProfile, CheckTarget, CheckTask, plan_checks};

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
            "maturin = \"1.11.0\"\n",
            "actionlint = \"1.7.12\"\n",
            "shellcheck = \"0.11.0\"\n",
            "\n",
            "[automation.actionlint-image]\n",
            "repository = \"rhysd/actionlint\"\n",
            "digest = \"sha256:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa\"\n",
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
    let plan = plan_checks(CheckTarget::Routine, CheckProfile::Fast);

    assert_eq!(
        plan.tasks,
        [
            CheckTask::RustFormat,
            CheckTask::RustLint,
            CheckTask::RustTests,
            CheckTask::Actionlint,
            CheckTask::AutomationPolicy,
        ]
    );
}

#[test]
fn all_check_plan_keeps_release_build_in_the_full_profile() {
    let fast_plan = plan_checks(CheckTarget::All, CheckProfile::Fast);
    let full_plan = plan_checks(CheckTarget::All, CheckProfile::Full);

    assert_eq!(
        fast_plan.tasks,
        [
            CheckTask::RustFormat,
            CheckTask::RustLint,
            CheckTask::RustTests,
            CheckTask::Actionlint,
            CheckTask::AutomationPolicy,
            CheckTask::DocsSync,
            CheckTask::ReleasePolicy,
        ]
    );
    assert_eq!(
        full_plan.tasks,
        [
            CheckTask::RustFormat,
            CheckTask::RustLint,
            CheckTask::RustTests,
            CheckTask::Actionlint,
            CheckTask::AutomationPolicy,
            CheckTask::DocsSync,
            CheckTask::ReleasePolicy,
            CheckTask::ReleaseBuild,
        ]
    );
}

#[test]
fn release_build_plan_clears_dist_then_builds_sdist_and_wheel() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_dev_tools_manifest(temp.path());
    let plan = tq_dev::release::plan(temp.path()).expect("release build plan should resolve");

    let details = plan
        .actions
        .iter()
        .map(|action| action.action.detail())
        .collect::<Vec<_>>();

    assert_eq!(
        details,
        [
            format!("remove {}", temp.path().join("dist").display()),
            "uv build --sdist".to_owned(),
            "uv run --isolated --with maturin==1.11.0 -- maturin build --release --locked \
             --manifest-path crates/tq-cli/Cargo.toml --bindings bin --out dist -i python"
                .to_owned(),
        ]
    );
}

#[test]
fn release_build_tool_requirements_are_exact_pins_from_manifest() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_dev_tools_manifest(temp.path());

    let tools = tq_dev::release::build_tool_requirements(temp.path())
        .expect("release build tools should resolve");

    assert_eq!(tools.maturin, "maturin==1.11.0");
    assert_eq!(tools.maturin_zig, "maturin[zig]==1.11.0");
    assert_eq!(
        tools.github_output(),
        "maturin=maturin==1.11.0\nmaturin_zig=maturin[zig]==1.11.0\n"
    );
}

#[test]
fn dependency_update_plan_updates_repository_owned_state_only() {
    let temp = tempfile::tempdir().expect("tempdir");
    write_dev_tools_manifest(temp.path());

    let plan = tq_dev::update::plan(temp.path()).expect("dependency update plan should build");
    let commands = plan
        .actions
        .iter()
        .filter_map(|action| match &action.action {
            Action::Command { invocation } => Some(invocation.display()),
            Action::ReplaceText { .. } | Action::RemovePath { .. } => None,
        })
        .collect::<Vec<_>>();

    assert_eq!(
        commands,
        [
            "uv lock --upgrade",
            "npm update",
            "uv run prek autoupdate --freeze",
        ]
    );
}

#[test]
fn setup_plan_installs_repository_owned_state_only() {
    let commands = tq_dev::setup::plan()
        .actions
        .iter()
        .map(|action| action.action.detail())
        .collect::<Vec<_>>();

    assert_eq!(
        commands,
        [
            "uv sync --locked",
            "npm ci --ignore-scripts",
            "uv run prek install --install-hooks",
        ]
    );
}
