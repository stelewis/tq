use tq_release::{DevCheckProfile, DevCheckTarget};

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
