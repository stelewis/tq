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
fn bounded_workflow_jobs_all_define_timeouts() {
    let external = include_str!("../../../.github/workflows/pinned-external-dependency-drift.yml");
    let maintenance = include_str!("../../../.github/workflows/rust-maintenance-tool-pins.yml");
    let copilot = include_str!("../../../.github/workflows/copilot-setup-steps.yml");

    assert!(external.contains("sync-drift-issue:\n    name: Sync pin drift issue\n    if: ${{ github.event_name == 'schedule' }}\n    needs: audit-pins\n    runs-on: ubuntu-latest\n    timeout-minutes: 5"));
    assert!(external.contains("fail-on-drift:\n    name: Fail on drift\n    if: ${{ needs.audit-pins.outputs.drift_detected == 'true' }}\n    needs: audit-pins\n    runs-on: ubuntu-latest\n    timeout-minutes: 1"));
    assert!(maintenance.contains("sync-maintenance-tool-pin-issue:\n    name: Sync maintenance tool pin issue\n    if: ${{ github.event_name == 'schedule' }}\n    needs: audit-maintenance-tool-pins\n    runs-on: ubuntu-latest\n    timeout-minutes: 5"));
    assert!(maintenance.contains("fail-on-drift:\n    name: Fail on drift\n    if: ${{ needs.audit-maintenance-tool-pins.outputs.drift_detected == 'true' }}\n    needs: audit-maintenance-tool-pins\n    runs-on: ubuntu-latest\n    timeout-minutes: 1"));
    assert!(
        copilot
            .contains("copilot-setup-steps:\n    runs-on: ubuntu-latest\n    timeout-minutes: 10")
    );
}
