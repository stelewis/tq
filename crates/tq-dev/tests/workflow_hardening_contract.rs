use std::fs;
use std::path::Path;

use tq_dev::workflow_policy::verify_workflow_hardening;

fn write(path: &Path, contents: &str) {
    fs::create_dir_all(path.parent().expect("fixture path must have a parent"))
        .expect("create fixture directory");
    fs::write(path, contents).expect("write fixture");
}

#[test]
fn docs_pages_scopes_write_permissions_to_deploy_job() {
    let workflow = include_str!("../../../.github/workflows/docs-pages.yml");
    let (global, jobs) = workflow
        .split_once("jobs:")
        .expect("workflow must define jobs");
    let deploy = jobs
        .split_once("  deploy:")
        .map(|(_, deploy)| deploy)
        .expect("workflow must define deploy job");

    assert!(global.contains("permissions:\n  contents: read"));
    assert!(!global.contains("pages: write"));
    assert!(!global.contains("id-token: write"));
    assert!(
        deploy.contains(
            "permissions:\n      contents: read\n      id-token: write\n      pages: write"
        )
    );
}

#[test]
fn docs_setup_pins_mise_and_disables_npm_lifecycle_scripts() {
    let action = include_str!("../../../.github/actions/setup-mise-docs/action.yml");

    assert!(action.contains("mise-version:"));
    assert!(action.contains("version: ${{ inputs.mise-version }}"));
    assert!(action.contains("npm ci --ignore-scripts"));
}

#[test]
fn every_workflow_and_job_satisfies_the_hardening_policy() {
    let repo_root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(Path::parent)
        .expect("tq-dev must be inside the workspace root");

    verify_workflow_hardening(repo_root).expect("all workflow jobs must satisfy policy");
    let ci = include_str!("../../../.github/workflows/ci.yml");
    assert!(ci.contains("cargo dev policy verify-automation --repo-root ."));
}

#[test]
fn new_workflow_jobs_fail_closed_without_timeout_or_with_global_write() {
    let temp = tempfile::tempdir().expect("tempdir");
    write(
        &temp.path().join(".github/workflows/new.yml"),
        concat!(
            "name: New workflow\n",
            "permissions:\n",
            "  contents: write\n",
            "jobs:\n",
            "  build:\n",
            "    runs-on: ubuntu-latest\n",
            "    steps:\n",
            "      - run: cargo test\n",
            "  deploy:\n",
            "    runs-on: ubuntu-latest\n",
            "    timeout-minutes: 0\n",
        ),
    );

    let error =
        verify_workflow_hardening(temp.path()).expect_err("unsafe new workflow must fail policy");
    let message = error.to_string();
    assert!(message.contains("workflow-global contents write permission is forbidden"));
    assert!(message.contains("job \"build\" must declare timeout-minutes"));
    assert!(message.contains("job \"deploy\" timeout-minutes must be a positive integer"));
}

#[test]
fn unsupported_or_empty_jobs_shape_fails_closed() {
    let temp = tempfile::tempdir().expect("tempdir");
    write(
        &temp.path().join(".github/workflows/empty.yml"),
        "name: Empty\npermissions: read-all\njobs: {}\n",
    );

    let error = verify_workflow_hardening(temp.path())
        .expect_err("workflow without parsed jobs must fail policy");
    assert!(error.to_string().contains("must declare at least one job"));
}

#[test]
fn composite_action_npm_installs_fail_closed_without_ignore_scripts() {
    let temp = tempfile::tempdir().expect("tempdir");
    write(
        &temp.path().join(".github/workflows/ci.yml"),
        concat!(
            "name: CI\n",
            "permissions: read-all\n",
            "jobs:\n",
            "  test:\n",
            "    runs-on: ubuntu-latest\n",
            "    timeout-minutes: 1\n",
        ),
    );
    write(
        &temp.path().join(".github/actions/setup/action.yml"),
        "runs:\n  using: composite\n  steps:\n    - shell: bash\n      run: npm ci\n",
    );

    let error =
        verify_workflow_hardening(temp.path()).expect_err("npm lifecycle scripts must fail policy");
    let message = error.to_string();
    assert!(message.contains(".github/actions/setup/action.yml"));
    assert!(message.contains("npm ci must include --ignore-scripts"));
}
